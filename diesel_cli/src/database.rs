use clap::ArgMatches;
use url::Url;
use diesel::expression::sql;
#[cfg(feature="postgres")]
use diesel::pg::PgConnection;
#[cfg(feature="sqlite")]
use diesel::sqlite::SqliteConnection;
#[cfg(feature="mysql")]
use diesel::mysql::MysqlConnection;
use diesel::types::Bool;
use diesel::*;

use database_error::{DatabaseError, DatabaseResult};

use std::error::Error;
use std::env;

enum Backend {
    #[cfg(feature="postgres")]
    Pg,
    #[cfg(feature="sqlite")]
    Sqlite,
    #[cfg(feature="mysql")]
    Mysql,
}

impl Backend {
    fn for_url(database_url: &str) -> Self {
        match database_url {
            #[cfg(feature="postgres")]
            _ if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") =>
                Backend::Pg,
            #[cfg(feature="mysql")]
            _ if database_url.starts_with("mysql://") =>
                Backend::Mysql,
            #[cfg(feature="sqlite")]
            _ => Backend::Sqlite,
            #[cfg(not(feature="sqlite"))]
            _ => {
                let mut available_schemes: Vec<&str> = Vec::new();

                // One of these will always be true, or you are compiling
                // diesel_cli without a backend. And why would you ever want to
                // do that?
                if cfg!(feature="postgres") {
                    available_schemes.push("`postgres://`");
                }
                if cfg!(feature="mysql") {
                    available_schemes.push("`mysql://`");
                }

                panic!("`{}` is not a valid database URL. It should start with {}",
                    database_url, available_schemes.join(" or "));
            },
        }
    }
}

pub enum InferConnection {
    #[cfg(feature="postgres")]
    Pg(PgConnection),
    #[cfg(feature="sqlite")]
    Sqlite(SqliteConnection),
    #[cfg(feature="mysql")]
    Mysql(MysqlConnection),
}

impl InferConnection {
    pub fn establish(database_url: &str) -> DatabaseResult<Self> {
        match Backend::for_url(database_url) {
            #[cfg(feature="postgres")]
            Backend::Pg => PgConnection::establish(database_url)
                .map(InferConnection::Pg),
            #[cfg(feature="sqlite")]
            Backend::Sqlite => SqliteConnection::establish(database_url)
                    .map(InferConnection::Sqlite),
            #[cfg(feature="mysql")]
            Backend::Mysql => MysqlConnection::establish(database_url)
                    .map(InferConnection::Mysql),
        }.map_err(Into::into)
    }
}

macro_rules! call_with_conn {
    (
        $database_url:expr,
        $($func:ident)::+
    ) => {
        call_with_conn!($database_url, $($func)::+ ())
    };

    (
        $database_url:expr,
        $($func:ident)::+ ($($args:expr),*)
    ) => {
        match ::database::InferConnection::establish(&$database_url).unwrap() {
            #[cfg(feature="postgres")]
            ::database::InferConnection::Pg(ref conn) => $($func)::+ (conn, $($args),*),
            #[cfg(feature="sqlite")]
            ::database::InferConnection::Sqlite(ref conn) => $($func)::+ (conn, $($args),*),
            #[cfg(feature="mysql")]
            ::database::InferConnection::Mysql(ref conn) => $($func)::+ (conn, $($args),*),
        }
    };
}

pub fn reset_database(args: &ArgMatches) -> DatabaseResult<()> {
    try!(drop_database(&database_url(args)));
    setup_database(args)
}

pub fn setup_database(args: &ArgMatches) -> DatabaseResult<()> {
    let database_url = database_url(args);

    try!(create_database_if_needed(&database_url));
    create_schema_table_and_run_migrations_if_needed(&database_url)
}

pub fn drop_database_command(args: &ArgMatches) -> DatabaseResult<()> {
    drop_database(&database_url(args))
}

/// Creates the database specified in the connection url. It returns an error
/// it it was unable to create the database.
fn create_database_if_needed(database_url: &str) -> DatabaseResult<()> {
    match Backend::for_url(database_url) {
        #[cfg(feature="postgres")]
        Backend::Pg => {
            if PgConnection::establish(database_url).is_err() {
                let (database, postgres_url) = change_database_of_url(database_url, "postgres");
                println!("Creating database: {}", database);
                let conn = try!(PgConnection::establish(&postgres_url));
                try!(conn.execute(&format!("CREATE DATABASE {}", database)));
            }
        },
        #[cfg(feature="sqlite")]
        Backend::Sqlite => {
            if !::std::path::Path::new(database_url).exists() {
                println!("Creating database: {}", database_url);
                try!(SqliteConnection::establish(database_url));
            }
        },
        #[cfg(feature="mysql")]
        Backend::Mysql => {
            if MysqlConnection::establish(database_url).is_err() {
                let (database, mysql_url) = change_database_of_url(database_url, "information_schema");
                println!("Creating database: {}", database);
                let conn = try!(MysqlConnection::establish(&mysql_url));
                try!(conn.execute(&format!("CREATE DATABASE {}", database)));
            }
        },
    }

    Ok(())
}

/// Creates the `__diesel_schema_migrations` table if it doesn't exist. If the
/// table didn't exist, it also runs any pending migrations. Returns a
/// `DatabaseError::ConnectionError` if it can't create the table, and exits
/// with a migration error if it can't run migrations.
fn create_schema_table_and_run_migrations_if_needed(database_url: &str)
    -> DatabaseResult<()>
{
    if !schema_table_exists(database_url).unwrap_or_else(handle_error) {
        try!(call_with_conn!(database_url, migrations::setup_database));
        call_with_conn!(database_url, migrations::run_pending_migrations).unwrap_or_else(handle_error);
    };
    Ok(())
}

/// Drops the database specified in the connection url. It returns an error
/// if it was unable to drop the database.
fn drop_database(database_url: &str) -> DatabaseResult<()> {
    match Backend::for_url(database_url) {
        #[cfg(feature="postgres")]
        Backend::Pg => {
            let (database, postgres_url) = change_database_of_url(database_url, "postgres");
            let conn = try!(PgConnection::establish(&postgres_url));
            if try!(pg_database_exists(&conn, &database)) {
                println!("Dropping database: {}", database);
                try!(conn.execute(&format!("DROP DATABASE IF EXISTS {}", database)));
            }
        },
        #[cfg(feature="sqlite")]
        Backend::Sqlite => {
            use std::path::Path;
            use std::fs;

            if Path::new(database_url).exists() {
                println!("Dropping database: {}", database_url);
                try!(fs::remove_file(&database_url));
            }
        },
        #[cfg(feature="mysql")]
        Backend::Mysql => {
            let (database, mysql_url) = change_database_of_url(database_url, "information_schema");
            let conn = try!(MysqlConnection::establish(&mysql_url));
            if try!(mysql_database_exists(&conn, &database)) {
                println!("Dropping database: {}", database);
                try!(conn.execute(&format!("DROP DATABASE IF EXISTS {}", database)));
            }
        },
    }
    Ok(())
}

#[cfg(feature="postgres")]
table! {
    pg_database (datname) {
        datname -> Text,
        datistemplate -> Bool,
    }
}

#[cfg(feature="postgres")]
fn pg_database_exists(conn: &PgConnection, database_name: &str) -> QueryResult<bool> {
    use self::pg_database::dsl::*;

    pg_database.select(datname) // here come dat name!!!! o shit waddup!!!!
        .filter(datname.eq(database_name))
        .filter(datistemplate.eq(false))
        .get_result::<String>(conn)
        .optional()
        .map(|x| x.is_some())
}

#[cfg(feature="mysql")]
table! {
    information_schema.schemata (schema_name) {
        schema_name -> Text,
    }
}

#[cfg(feature="mysql")]
fn mysql_database_exists(conn: &MysqlConnection, database_name: &str) -> QueryResult<bool> {
    use self::schemata::dsl::*;

    schemata.select(schema_name)
        .filter(schema_name.eq(database_name))
        .get_result::<String>(conn)
        .optional()
        .map(|x| x.is_some())
}

/// Returns true if the `__diesel_schema_migrations` table exists in the
/// database we connect to, returns false if it does not.
pub fn schema_table_exists(database_url: &str) -> DatabaseResult<bool> {
    match InferConnection::establish(database_url).unwrap() {
        #[cfg(feature="postgres")]
        InferConnection::Pg(conn) => {
            select(sql::<Bool>("EXISTS \
                    (SELECT 1 \
                     FROM information_schema.tables \
                     WHERE table_name = '__diesel_schema_migrations')"))
                .get_result(&conn)
        },
        #[cfg(feature="sqlite")]
        InferConnection::Sqlite(conn) => {
            select(sql::<Bool>("EXISTS \
                    (SELECT 1 \
                     FROM sqlite_master \
                     WHERE type = 'table' \
                     AND name = '__diesel_schema_migrations')"))
                .get_result(&conn)
        },
        #[cfg(feature="mysql")]
        InferConnection::Mysql(conn) => {
            select(sql::<Bool>("EXISTS \
                    (SELECT 1 \
                     FROM information_schema.tables \
                     WHERE table_name = '__diesel_schema_migrations'
                     AND table_schema = DATABASE())"))
                .get_result(&conn)
        },
    }.map_err(Into::into)
}

pub fn database_url(matches: &ArgMatches) -> String {
    matches.value_of("DATABASE_URL")
        .map(|s| s.into())
        .or_else(|| env::var("DATABASE_URL").ok())
        .unwrap_or_else(|| {
            handle_error(DatabaseError::DatabaseUrlMissing)
        })
}

#[cfg(any(feature="postgres", feature="mysql"))]
fn change_database_of_url(database_url: &str, default_database: &str) -> (String, String) {
    // This method accepts a MySQL or Postgres connection string that conforms to URL BNF grammar
    // only (and not the database connection string grammars) and returns the name of the database
    // and a new connection string to the database specified by the default_database argument. If
    // the database name is not specified in the connection string then we default to an empty
    // string. NOTE: This function does not validate connection strings (look at the signature, no
    // Option or Result is there).

    // TODO: handle unix connection strings with percent encoded paths such as:
    //  postgres://%2Fvar%2Flib/dbname
    let mut url = Url::parse(database_url).expect("failed to parse connection string");
    let database: String = url.path().trim_left_matches('/').to_string();
    url.set_path(default_database);

    (database, url.into_string())
}

#[cfg_attr(feature="clippy", allow(needless_pass_by_value))]
fn handle_error<E: Error, T>(error: E) -> T {
    println!("{}", error);
    ::std::process::exit(1);
}

#[cfg(all(test, any(feature="postgres", feature="mysql")))]
mod tests {
    use super::change_database_of_url;

    #[test]
    fn split_pg_connection_string_returns_postgres_url_and_database() {
        // format: (original_string, dbname, expected_changed_string)
        let test_cases: [(&'static str, &'static str, &'static str); 10] = [
            ("postgresql://", "", "postgresql:///postgres"),
            ("postgresql://localhost", "" ,"postgresql://localhost/postgres"),
            ("postgresql://localhost:5433", "", "postgresql://localhost:5433/postgres"),
            ("postgresql://localhost/mydb", "mydb", "postgresql://localhost/postgres"),
            ("postgresql://user@localhost", "","postgresql://user@localhost/postgres"),
            ("postgresql://user:secret@localhost", "", "postgresql://user:secret@localhost/postgres"),
            ("postgresql://other@localhost/otherdb?connect_timeout=10&application_name=myapp", "otherdb", "postgresql://other@localhost/postgres?connect_timeout=10&application_name=myapp" ),
            ("postgresql:///mydb?host=localhost&port=5433", "mydb", "postgresql:///postgres?host=localhost&port=5433"),
            ("postgresql://[2001:db8::1234]/database", "database", "postgresql://[2001:db8::1234]/postgres"),
            ("postgresql:///dbname?host=/var/lib/postgresql", "dbname", "postgresql:///postgres?host=/var/lib/postgresql"),
            // TODO: Support opaque origins
            //("postgresql://%2Fvar%2Flib%2Fpostgresql/dbname", "dbname", "postgresql://%2Fvar%2Flib%2Fpostgresql/postgres"),
        ];
        for &(database_url, database, postgres_url) in &test_cases {
            assert_eq!((database.to_owned(), postgres_url.to_owned()), change_database_of_url(database_url, "postgres"));
        }
    }

    #[test]
    fn split_mysql_connection_string_returns_mysql_url_and_database() {
        // format: (original_string, dbname, expected_changed_string)
        let test_cases: [(&'static str, &'static str, &'static str); 8] = [
            ("mysql://", "", "mysql:///information_schema"),
            ("mysql://localhost", "" ,"mysql://localhost/information_schema"),
            ("mysql://localhost:5433", "", "mysql://localhost:5433/information_schema"),
            ("mysql://localhost/mydb", "mydb", "mysql://localhost/information_schema"),
            ("mysql://user@localhost", "","mysql://user@localhost/information_schema"),
            ("mysql://user:secret@localhost", "", "mysql://user:secret@localhost/information_schema"),
            ("mysql://other@localhost/otherdb", "otherdb", "mysql://other@localhost/information_schema"),
            ("mysql://other@localhost:1122/otherdb", "otherdb", "mysql://other@localhost:1122/information_schema"),
        ];
        for &(database_url, database, mysql_url) in &test_cases {
            assert_eq!((database.to_owned(), mysql_url.to_owned()), change_database_of_url(database_url, "information_schema"));
        }
    }
}
