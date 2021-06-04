use std::marker::PhantomData;

use crate::backend::{Backend, SupportsDefaultKeyword};
use crate::expression::grouped::Grouped;
use crate::expression::{AppearsOnTable, Expression};
use crate::query_builder::{
    AstPass, InsertStatement, QueryFragment, QueryId, UndecoratedInsertRecord, ValuesClause,
};
use crate::query_source::{Column, Table};
use crate::result::QueryResult;

/// Represents that a structure can be used to insert a new row into the
/// database. This is automatically implemented for `&[T]` and `&Vec<T>` for
/// inserting more than one record.
///
/// This trait can be [derived](derive@Insertable)
pub trait Insertable<T> {
    /// The `VALUES` clause to insert these records
    ///
    /// The types used here are generally internal to Diesel.
    /// Implementations of this trait should use the `Values`
    /// type of other `Insertable` types.
    /// For example `<diesel::dsl::Eq<column, &str> as Insertable<table>>::Values`.
    type Values;

    /// Construct `Self::Values`
    ///
    /// Implementations of this trait typically call `.values`
    /// on other `Insertable` types.
    fn values(self) -> Self::Values;

    /// Insert `self` into a given table.
    ///
    /// `foo.insert_into(table)` is identical to `insert_into(table).values(foo)`.
    /// However, when inserting from a select statement,
    /// this form is generally preferred.
    ///
    /// # Example
    ///
    /// ```rust
    /// # include!("doctest_setup.rs");
    /// #
    /// # fn main() {
    /// #     run_test().unwrap();
    /// # }
    /// #
    /// # fn run_test() -> QueryResult<()> {
    /// #     use schema::{posts, users};
    /// #     let conn = &mut establish_connection();
    /// #     diesel::delete(posts::table).execute(conn)?;
    /// users::table
    ///     .select((
    ///         users::name.concat("'s First Post"),
    ///         users::id,
    ///     ))
    ///     .insert_into(posts::table)
    ///     .into_columns((posts::title, posts::user_id))
    ///     .execute(conn)?;
    ///
    /// let inserted_posts = posts::table
    ///     .select(posts::title)
    ///     .load::<String>(conn)?;
    /// let expected = vec!["Sean's First Post", "Tess's First Post"];
    /// assert_eq!(expected, inserted_posts);
    /// #     Ok(())
    /// # }
    /// ```
    fn insert_into(self, table: T) -> InsertStatement<T, Self::Values>
    where
        Self: Sized,
    {
        crate::insert_into(table).values(self)
    }
}

#[doc(inline)]
pub use diesel_derives::Insertable;

pub trait CanInsertInSingleQuery<DB: Backend> {
    /// How many rows will this query insert?
    ///
    /// This function should only return `None` when the query is valid on all
    /// backends, regardless of how many rows get inserted.
    fn rows_to_insert(&self) -> Option<usize>;
}

impl<'a, T, DB> CanInsertInSingleQuery<DB> for &'a T
where
    T: ?Sized + CanInsertInSingleQuery<DB>,
    DB: Backend,
{
    fn rows_to_insert(&self) -> Option<usize> {
        (*self).rows_to_insert()
    }
}

impl<'a, T, Tab, DB> CanInsertInSingleQuery<DB> for BatchInsert<'a, T, Tab>
where
    DB: Backend + SupportsDefaultKeyword,
{
    fn rows_to_insert(&self) -> Option<usize> {
        Some(self.records.len())
    }
}

impl<T, Table, DB> CanInsertInSingleQuery<DB> for OwnedBatchInsert<T, Table>
where
    DB: Backend + SupportsDefaultKeyword,
{
    fn rows_to_insert(&self) -> Option<usize> {
        Some(self.values.len())
    }
}

impl<T, Table, DB, const N: usize> CanInsertInSingleQuery<DB> for StaticBatchInsert<T, Table, N>
where
    DB: Backend + SupportsDefaultKeyword,
{
    fn rows_to_insert(&self) -> Option<usize> {
        Some(N)
    }
}

impl<T, U, DB> CanInsertInSingleQuery<DB> for ColumnInsertValue<T, U>
where
    DB: Backend,
{
    fn rows_to_insert(&self) -> Option<usize> {
        Some(1)
    }
}

impl<V, DB> CanInsertInSingleQuery<DB> for DefaultableColumnInsertValue<V>
where
    DB: Backend,
    V: CanInsertInSingleQuery<DB>,
{
    fn rows_to_insert(&self) -> Option<usize> {
        Some(1)
    }
}

pub trait InsertValues<T: Table, DB: Backend>: QueryFragment<DB> {
    fn column_names(&self, out: AstPass<DB>) -> QueryResult<()>;
}

#[derive(Debug, Copy, Clone, QueryId)]
#[doc(hidden)]
pub struct ColumnInsertValue<Col, Expr> {
    col: Col,
    expr: Expr,
}

impl<Col, Expr> ColumnInsertValue<Col, Expr> {
    pub(crate) fn new(col: Col, expr: Expr) -> Self {
        Self { col, expr }
    }
}

#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub enum DefaultableColumnInsertValue<T> {
    Expression(T),
    Default,
}

impl<T> QueryId for DefaultableColumnInsertValue<T> {
    type QueryId = ();
    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<T> Default for DefaultableColumnInsertValue<T> {
    fn default() -> Self {
        DefaultableColumnInsertValue::Default
    }
}

impl<Col, Expr, DB> InsertValues<Col::Table, DB>
    for DefaultableColumnInsertValue<ColumnInsertValue<Col, Expr>>
where
    DB: Backend + SupportsDefaultKeyword,
    Col: Column,
    Expr: Expression<SqlType = Col::SqlType> + AppearsOnTable<()>,
    Self: QueryFragment<DB>,
{
    fn column_names(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        out.push_identifier(Col::NAME)?;
        Ok(())
    }
}

impl<Col, Expr, DB> InsertValues<Col::Table, DB> for ColumnInsertValue<Col, Expr>
where
    DB: Backend,
    Col: Column,
    Expr: Expression<SqlType = Col::SqlType> + AppearsOnTable<()>,
    Self: QueryFragment<DB>,
{
    fn column_names(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        out.push_identifier(Col::NAME)?;
        Ok(())
    }
}

impl<Expr, DB> QueryFragment<DB> for DefaultableColumnInsertValue<Expr>
where
    DB: Backend + SupportsDefaultKeyword,
    Expr: QueryFragment<DB>,
{
    fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        if let Self::Expression(ref inner) = *self {
            inner.walk_ast(out.reborrow())?;
        } else {
            out.push_sql("DEFAULT");
        }
        Ok(())
    }
}

impl<Col, Expr, DB> QueryFragment<DB> for ColumnInsertValue<Col, Expr>
where
    DB: Backend,
    Expr: QueryFragment<DB>,
{
    fn walk_ast(&self, pass: AstPass<DB>) -> QueryResult<()> {
        self.expr.walk_ast(pass)
    }
}

#[cfg(feature = "sqlite")]
impl<Col, Expr> InsertValues<Col::Table, crate::sqlite::Sqlite>
    for DefaultableColumnInsertValue<ColumnInsertValue<Col, Expr>>
where
    Col: Column,
    Expr: Expression<SqlType = Col::SqlType> + AppearsOnTable<()>,
    Self: QueryFragment<crate::sqlite::Sqlite>,
{
    fn column_names(&self, mut out: AstPass<crate::sqlite::Sqlite>) -> QueryResult<()> {
        if let Self::Expression(..) = *self {
            out.push_identifier(Col::NAME)?;
        }
        Ok(())
    }
}

#[cfg(feature = "sqlite")]
impl<Col, Expr> QueryFragment<crate::sqlite::Sqlite>
    for DefaultableColumnInsertValue<ColumnInsertValue<Col, Expr>>
where
    Expr: QueryFragment<crate::sqlite::Sqlite>,
{
    fn walk_ast(&self, mut out: AstPass<crate::sqlite::Sqlite>) -> QueryResult<()> {
        if let Self::Expression(ref inner) = *self {
            inner.walk_ast(out.reborrow())?;
        }
        Ok(())
    }
}

impl<'a, T, Tab> Insertable<Tab> for &'a [T]
where
    &'a T: UndecoratedInsertRecord<Tab>,
{
    type Values = BatchInsert<'a, T, Tab>;

    fn values(self) -> Self::Values {
        BatchInsert {
            records: self,
            _marker: PhantomData,
        }
    }
}

impl<'a, T, Tab> Insertable<Tab> for &'a Vec<T>
where
    &'a [T]: Insertable<Tab>,
{
    type Values = <&'a [T] as Insertable<Tab>>::Values;

    fn values(self) -> Self::Values {
        (&**self).values()
    }
}

impl<T, Tab> Insertable<Tab> for Vec<T>
where
    T: Insertable<Tab> + UndecoratedInsertRecord<Tab>,
{
    type Values = OwnedBatchInsert<T::Values, Tab>;

    fn values(self) -> Self::Values {
        OwnedBatchInsert {
            values: self.into_iter().map(Insertable::values).collect(),
            _marker: PhantomData,
        }
    }
}

impl<T, Tab, const N: usize> Insertable<Tab> for [T; N]
where
    T: Insertable<Tab> + UndecoratedInsertRecord<Tab>,
{
    type Values = StaticBatchInsert<T::Values, Tab, N>;

    fn values(self) -> Self::Values {
        use std::mem::{self, MaybeUninit};

        struct DropGuard<T> {
            base_ptr: *mut T,
            initialized_count: usize,
        }

        impl<T> Drop for DropGuard<T> {
            fn drop(&mut self) {
                unsafe {
                    // # Safety
                    //
                    //   initialized_count is increased in such a way that
                    //   it this only points to initilaized memory cells
                    std::ptr::drop_in_place(std::slice::from_raw_parts_mut(
                        self.base_ptr,
                        self.initialized_count,
                    ));
                }
            }
        }

        let mut uninit_values: [MaybeUninit<T::Values>; N] = unsafe {
            // # Safety
            //
            // Create an uninitialized array of `MaybeUninit`. The `assume_init` is
            // safe because the type we are claiming to have initialized here is a
            // bunch of `MaybeUninit`s, which do not require initialization.

            MaybeUninit::uninit().assume_init()
        };

        let mut drop_guard = DropGuard {
            base_ptr: &mut uninit_values as *mut _ as *mut T::Values,
            initialized_count: 0,
        };

        for (value, target) in std::array::IntoIter::new(self).zip(&mut uninit_values) {
            *target = MaybeUninit::new(Insertable::values(value));
            drop_guard.initialized_count += 1;
        }

        // no panic after this point that could leak memory
        mem::forget(drop_guard);

        // The whole array is initialized at this point, unfortunally there is
        // no way to signal that using the `MaybeUninit` api or even `transmute`
        // See https://github.com/rust-lang/rust/issues/61956 for details
        let values = &mut uninit_values as *mut _ as *mut [T::Values; N];
        // # Safety
        // The `MaybeUninit` is just a transparent wrapper around `T`
        // so alignment and size are the same according to the std-lib documentation
        let values = unsafe { values.read() };
        mem::forget(uninit_values);

        StaticBatchInsert {
            values,
            _marker: PhantomData,
        }
    }
}

impl<T, V, Tab> Insertable<Tab> for Option<T>
where
    T: Insertable<Tab, Values = ValuesClause<V, Tab>>,
{
    type Values = ValuesClause<DefaultableColumnInsertValue<V>, Tab>;

    fn values(self) -> Self::Values {
        ValuesClause::new(
            self.map(|v| DefaultableColumnInsertValue::Expression(Insertable::values(v).values))
                .unwrap_or_default(),
        )
    }
}

impl<'a, T, Tab> Insertable<Tab> for &'a Option<T>
where
    Option<&'a T>: Insertable<Tab>,
{
    type Values = <Option<&'a T> as Insertable<Tab>>::Values;

    fn values(self) -> Self::Values {
        self.as_ref().values()
    }
}

impl<L, R, Tab> Insertable<Tab> for Grouped<crate::expression::operators::Eq<L, R>>
where
    crate::expression::operators::Eq<L, R>: Insertable<Tab>,
{
    type Values = <crate::expression::operators::Eq<L, R> as Insertable<Tab>>::Values;

    fn values(self) -> Self::Values {
        self.0.values()
    }
}

impl<'a, L, R, Tab> Insertable<Tab> for &'a Grouped<crate::expression::operators::Eq<L, R>>
where
    &'a crate::expression::operators::Eq<L, R>: Insertable<Tab>,
{
    type Values = <&'a crate::expression::operators::Eq<L, R> as Insertable<Tab>>::Values;

    fn values(self) -> Self::Values {
        self.0.values()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BatchInsert<'a, T: 'a, Tab> {
    pub records: &'a [T],
    _marker: PhantomData<Tab>,
}

impl<'a, T, Tab> QueryId for BatchInsert<'a, T, Tab> {
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<'a, T, Tab, Inner, DB> QueryFragment<DB> for BatchInsert<'a, T, Tab>
where
    DB: Backend + SupportsDefaultKeyword,
    &'a T: Insertable<Tab, Values = ValuesClause<Inner, Tab>>,
    ValuesClause<Inner, Tab>: QueryFragment<DB>,
    Inner: QueryFragment<DB>,
{
    fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();

        let mut records = self.records.iter().map(Insertable::values);
        if let Some(record) = records.next() {
            record.walk_ast(out.reborrow())?;
        }
        for record in records {
            out.push_sql(", (");
            record.values.walk_ast(out.reborrow())?;
            out.push_sql(")");
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct OwnedBatchInsert<V, Tab> {
    pub values: Vec<V>,
    _marker: PhantomData<Tab>,
}

impl<Inner, Tab> QueryId for OwnedBatchInsert<ValuesClause<Inner, Tab>, Tab> {
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<Tab, DB, Inner> QueryFragment<DB> for OwnedBatchInsert<ValuesClause<Inner, Tab>, Tab>
where
    DB: Backend + SupportsDefaultKeyword,
    ValuesClause<Inner, Tab>: QueryFragment<DB>,
    Inner: QueryFragment<DB>,
{
    fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();

        let mut values = self.values.iter();
        if let Some(value) = values.next() {
            value.walk_ast(out.reborrow())?;
        }
        for value in values {
            out.push_sql(", (");
            value.values.walk_ast(out.reborrow())?;
            out.push_sql(")");
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct StaticBatchInsert<V, Tab, const N: usize> {
    pub values: [V; N],
    _marker: PhantomData<Tab>,
}

impl<Inner: 'static, Tab: 'static, const N: usize> QueryId
    for StaticBatchInsert<ValuesClause<Inner, Tab>, Tab, N>
{
    type QueryId = [ValuesClause<Inner, Tab>; N];
}

impl<Tab, DB, Inner, const N: usize> QueryFragment<DB>
    for StaticBatchInsert<ValuesClause<Inner, Tab>, Tab, N>
where
    DB: Backend + SupportsDefaultKeyword,
    ValuesClause<Inner, Tab>: QueryFragment<DB>,
    Inner: QueryFragment<DB>,
{
    fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        let mut values = self.values.iter();
        if let Some(value) = values.next() {
            value.walk_ast(out.reborrow())?;
        }
        for value in values {
            out.push_sql(", (");
            value.values.walk_ast(out.reborrow())?;
            out.push_sql(")");
        }
        Ok(())
    }
}
