//! Connection pooling via r2d2.
//!
//! Note: This module requires enabling the `r2d2` feature

extern crate r2d2;

pub use self::r2d2::*;

/// A re-export of [`r2d2::Error`], which is only used by methods on [`r2d2::Pool`].
///
/// [`r2d2::Error`]: ../../r2d2/struct.Error.html
/// [`r2d2::Pool`]: ../../r2d2/struct.Pool.html
pub type PoolError = self::r2d2::Error;

use std::convert::Into;
use std::fmt;
use std::marker::PhantomData;

use crate::backend::UsesAnsiSavepointSyntax;
use crate::connection::{AnsiTransactionManager, SimpleConnection};
use crate::deserialize::{Queryable, QueryableByName};
use crate::prelude::*;
use crate::query_builder::{AsQuery, QueryFragment, QueryId};
use crate::sql_types::HasSqlType;

/// An r2d2 connection manager for use with Diesel.
///
/// See the [r2d2 documentation] for usage examples.
///
/// [r2d2 documentation]: ../../r2d2
#[derive(Debug, Clone)]
pub struct ConnectionManager<T> {
    database_url: String,
    _marker: PhantomData<fn() -> T>,
}

impl<T> ConnectionManager<T> {
    /// Returns a new connection manager,
    /// which establishes connections to the given database URL.
    pub fn new<S: Into<String>>(database_url: S) -> Self {
        ConnectionManager {
            database_url: database_url.into(),
            _marker: PhantomData,
        }
    }
}

/// The error used when managing connections with `r2d2`.
#[derive(Debug)]
pub enum Error {
    /// An error occurred establishing the connection
    ConnectionError(ConnectionError),

    /// An error occurred pinging the database
    QueryError(crate::result::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ConnectionError(ref e) => e.fmt(f),
            Error::QueryError(ref e) => e.fmt(f),
        }
    }
}

impl ::std::error::Error for Error {}

/// A trait indicating a connection could be used inside a r2d2 pool
pub trait R2D2Connection: Connection {
    /// Check if a connection is still valid
    fn ping(&self) -> QueryResult<()>;
}

#[cfg(feature = "postgres")]
impl R2D2Connection for crate::pg::PgConnection {
    fn ping(&self) -> QueryResult<()> {
        self.execute("SELECT 1").map(|_| ())
    }
}

#[cfg(feature = "mysql")]
impl R2D2Connection for crate::mysql::MysqlConnection {
    fn ping(&self) -> QueryResult<()> {
        self.execute("SELECT 1").map(|_| ())
    }
}

#[cfg(feature = "sqlite")]
impl R2D2Connection for crate::sqlite::SqliteConnection {
    fn ping(&self) -> QueryResult<()> {
        self.execute("SELECT 1").map(|_| ())
    }
}

impl<T> ManageConnection for ConnectionManager<T>
where
    T: R2D2Connection + Send + 'static,
{
    type Connection = T;
    type Error = Error;

    fn connect(&self) -> Result<T, Error> {
        T::establish(&self.database_url).map_err(Error::ConnectionError)
    }

    fn is_valid(&self, conn: &mut T) -> Result<(), Error> {
        conn.ping().map_err(Error::QueryError)
    }

    fn has_broken(&self, _conn: &mut T) -> bool {
        false
    }
}

impl<M> SimpleConnection for PooledConnection<M>
where
    M: ManageConnection,
    M::Connection: R2D2Connection + Send + 'static,
{
    fn batch_execute(&self, query: &str) -> QueryResult<()> {
        (&**self).batch_execute(query)
    }
}

impl<M> Connection for PooledConnection<M>
where
    M: ManageConnection,
    M::Connection:
        Connection<TransactionManager = AnsiTransactionManager> + R2D2Connection + Send + 'static,
    <M::Connection as Connection>::Backend: UsesAnsiSavepointSyntax,
{
    type Backend = <M::Connection as Connection>::Backend;
    type TransactionManager = <M::Connection as Connection>::TransactionManager;

    fn establish(_: &str) -> ConnectionResult<Self> {
        Err(ConnectionError::BadConnection(String::from(
            "Cannot directly establish a pooled connection",
        )))
    }

    fn execute(&self, query: &str) -> QueryResult<usize> {
        (&**self).execute(query)
    }

    fn query_by_index<T, U>(&self, source: T) -> QueryResult<Vec<U>>
    where
        T: AsQuery,
        T::Query: QueryFragment<Self::Backend> + QueryId,
        Self::Backend: HasSqlType<T::SqlType>,
        U: Queryable<T::SqlType, Self::Backend>,
    {
        (&**self).query_by_index(source)
    }

    fn query_by_name<T, U>(&self, source: &T) -> QueryResult<Vec<U>>
    where
        T: QueryFragment<Self::Backend> + QueryId,
        U: QueryableByName<Self::Backend>,
    {
        (&**self).query_by_name(source)
    }

    fn execute_returning_count<T>(&self, source: &T) -> QueryResult<usize>
    where
        T: QueryFragment<Self::Backend> + QueryId,
    {
        (&**self).execute_returning_count(source)
    }

    fn transaction_manager(&self) -> &Self::TransactionManager {
        (&**self).transaction_manager()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::thread;

    use crate::r2d2::*;
    use crate::test_helpers::*;

    #[test]
    fn establish_basic_connection() {
        let manager = ConnectionManager::<TestConnection>::new(database_url());
        let pool = Arc::new(Pool::builder().max_size(2).build(manager).unwrap());

        let (s1, r1) = mpsc::channel();
        let (s2, r2) = mpsc::channel();

        let pool1 = Arc::clone(&pool);
        let t1 = thread::spawn(move || {
            let conn = pool1.get().unwrap();
            s1.send(()).unwrap();
            r2.recv().unwrap();
            drop(conn);
        });

        let pool2 = Arc::clone(&pool);
        let t2 = thread::spawn(move || {
            let conn = pool2.get().unwrap();
            s2.send(()).unwrap();
            r1.recv().unwrap();
            drop(conn);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        pool.get().unwrap();
    }

    #[test]
    fn is_valid() {
        let manager = ConnectionManager::<TestConnection>::new(database_url());
        let pool = Pool::builder()
            .max_size(1)
            .test_on_check_out(true)
            .build(manager)
            .unwrap();

        pool.get().unwrap();
    }

    #[test]
    fn pooled_connection_impls_connection() {
        use crate::select;
        use crate::sql_types::Text;

        let manager = ConnectionManager::<TestConnection>::new(database_url());
        let pool = Pool::builder()
            .max_size(1)
            .test_on_check_out(true)
            .build(manager)
            .unwrap();
        let conn = pool.get().unwrap();

        let query = select("foo".into_sql::<Text>());
        assert_eq!("foo", query.get_result::<String>(&conn).unwrap());
    }
}
