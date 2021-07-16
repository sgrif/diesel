use super::{InsertStatement, ValuesClause};
use crate::connection::Connection;
use crate::insertable::{BatchInsert, Insertable, OwnedBatchInsert, StaticBatchInsert};
use crate::query_builder::{DebugQuery, QueryFragment, QueryId};
use crate::query_dsl::methods::ExecuteDsl;
use crate::query_dsl::RunQueryDsl;
use crate::sqlite::Sqlite;
use crate::QueryResult;
use std::fmt::{self, Debug, Display};

impl<'a, T, U, Op, C> ExecuteDsl<C, Sqlite> for InsertStatement<T, BatchInsert<'a, U, T>, Op>
where
    C: Connection<Backend = Sqlite>,
    &'a U: Insertable<T>,
    InsertStatement<T, <&'a U as Insertable<T>>::Values, Op>: QueryFragment<Sqlite>,
    <&'a U as Insertable<T>>::Values: QueryId,
    T: Copy + QueryId,
    Op: Copy + QueryId,
{
    fn execute(query: Self, conn: &mut C) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let mut result = 0;
            for record in query.records.records {
                result += InsertStatement::new(
                    query.target,
                    record.values(),
                    query.operator,
                    query.returning,
                )
                .execute(conn)?;
            }
            Ok(result)
        })
    }
}

impl<'a, T, U, Op> Display for DebugQuery<'a, InsertStatement<T, BatchInsert<'a, U, T>, Op>, Sqlite>
where
    &'a U: Insertable<T>,
    for<'b> DebugQuery<'b, InsertStatement<T, <&'a U as Insertable<T>>::Values, Op>, Sqlite>:
        Display,
    T: Copy,
    Op: Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "BEGIN;")?;
        for record in self.query.records.records {
            let stmt = InsertStatement::new(
                self.query.target,
                record.values(),
                self.query.operator,
                self.query.returning,
            );

            writeln!(f, "{}", crate::debug_query::<Sqlite, _>(&stmt))?;
        }
        writeln!(f, "COMMIT;")?;
        Ok(())
    }
}

impl<'a, T, U, Op> Debug for DebugQuery<'a, InsertStatement<T, BatchInsert<'a, U, T>, Op>, Sqlite>
where
    &'a U: Insertable<T>,
    for<'b> DebugQuery<'b, InsertStatement<T, <&'a U as Insertable<T>>::Values, Op>, Sqlite>:
        Display,
    T: Copy,
    Op: Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut statements = Vec::with_capacity(self.query.records.records.len() + 2);
        statements.push("BEGIN".into());
        for record in self.query.records.records {
            let stmt = InsertStatement::new(
                self.query.target,
                record.values(),
                self.query.operator,
                self.query.returning,
            );
            statements.push(format!("{}", crate::debug_query::<Sqlite, _>(&stmt)));
        }
        statements.push("COMMIT".into());

        f.debug_struct("Query")
            .field("sql", &statements)
            .field("binds", &[] as &[i32; 0])
            .finish()
    }
}

impl<T, U, Op, C, const N: usize> ExecuteDsl<C, Sqlite>
    for InsertStatement<T, StaticBatchInsert<ValuesClause<U, T>, T, N>, Op>
where
    C: Connection<Backend = Sqlite>,
    for<'a> InsertStatement<T, &'a ValuesClause<U, T>, Op>: QueryFragment<Sqlite>,
    ValuesClause<U, T>: QueryId,
    T: Copy + QueryId,
    Op: Copy + QueryId,
{
    fn execute(query: Self, conn: &mut C) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let mut result = 0;
            for value in &query.records.values {
                result +=
                    InsertStatement::new(query.target, value, query.operator, query.returning)
                        .execute(conn)?;
            }
            Ok(result)
        })
    }
}

impl<'a, T, U, Op, const N: usize> Display
    for DebugQuery<'a, InsertStatement<T, StaticBatchInsert<ValuesClause<U, T>, T, N>, Op>, Sqlite>
where
    for<'b> DebugQuery<'b, InsertStatement<T, &'b ValuesClause<U, T>, Op>, Sqlite>: Display,
    T: Copy,
    Op: Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "BEGIN;")?;
        for value in &self.query.records.values {
            let stmt = InsertStatement::new(
                self.query.target,
                value,
                self.query.operator,
                self.query.returning,
            );

            writeln!(f, "{}", crate::debug_query::<Sqlite, _>(&stmt))?;
        }
        writeln!(f, "COMMIT;")?;
        Ok(())
    }
}

impl<'a, T, U, Op, const N: usize> Debug
    for DebugQuery<'a, InsertStatement<T, StaticBatchInsert<ValuesClause<U, T>, T, N>, Op>, Sqlite>
where
    for<'b> DebugQuery<'b, InsertStatement<T, &'b ValuesClause<U, T>, Op>, Sqlite>: Display,
    T: Copy,
    Op: Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut statements = Vec::with_capacity(self.query.records.values.len() + 2);
        statements.push("BEGIN".into());

        for value in &self.query.records.values {
            let stmt = InsertStatement::new(
                self.query.target,
                value,
                self.query.operator,
                self.query.returning,
            );
            statements.push(format!("{}", crate::debug_query::<Sqlite, _>(&stmt)));
        }
        statements.push("COMMIT".into());

        f.debug_struct("Query")
            .field("sql", &statements)
            .field("binds", &[] as &[i32; 0])
            .finish()
    }
}

impl<T, U, Op, C> ExecuteDsl<C, Sqlite>
    for InsertStatement<T, OwnedBatchInsert<ValuesClause<U, T>, T>, Op>
where
    C: Connection<Backend = Sqlite>,
    InsertStatement<T, ValuesClause<U, T>, Op>: QueryFragment<Sqlite>,
    ValuesClause<U, T>: QueryId,
    T: Copy + QueryId,
    Op: Copy + QueryId,
{
    fn execute(query: Self, conn: &mut C) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let mut result = 0;
            for value in query.records.values {
                result +=
                    InsertStatement::new(query.target, value, query.operator, query.returning)
                        .execute(conn)?;
            }
            Ok(result)
        })
    }
}

impl<'a, T, U, Op> Display
    for DebugQuery<'a, InsertStatement<T, OwnedBatchInsert<ValuesClause<U, T>, T>, Op>, Sqlite>
where
    for<'b> DebugQuery<'b, InsertStatement<T, &'b ValuesClause<U, T>, Op>, Sqlite>: Display,
    T: Copy,
    Op: Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "BEGIN;")?;
        for value in &self.query.records.values {
            let stmt = InsertStatement::new(
                self.query.target,
                value,
                self.query.operator,
                self.query.returning,
            );

            writeln!(f, "{}", crate::debug_query::<Sqlite, _>(&stmt))?;
        }
        writeln!(f, "COMMIT;")?;
        Ok(())
    }
}

impl<'a, T, U, Op> Debug
    for DebugQuery<'a, InsertStatement<T, OwnedBatchInsert<ValuesClause<U, T>, T>, Op>, Sqlite>
where
    for<'b> DebugQuery<'b, InsertStatement<T, &'b ValuesClause<U, T>, Op>, Sqlite>: Display,
    T: Copy,
    Op: Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut statements = Vec::with_capacity(self.query.records.values.len() + 2);
        statements.push("BEGIN".into());

        for value in &self.query.records.values {
            let stmt = InsertStatement::new(
                self.query.target,
                value,
                self.query.operator,
                self.query.returning,
            );
            statements.push(format!("{}", crate::debug_query::<Sqlite, _>(&stmt)));
        }
        statements.push("COMMIT".into());

        f.debug_struct("Query")
            .field("sql", &statements)
            .field("binds", &[] as &[i32; 0])
            .finish()
    }
}
