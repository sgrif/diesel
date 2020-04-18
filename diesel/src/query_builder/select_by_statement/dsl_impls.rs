// use super::BoxedSelectByStatement;
use crate::associations::HasTable;
// use crate::backend::Backend;
use crate::dsl::AsExprOf;
// use crate::expression::nullable::Nullable;
use crate::expression::*;
use crate::insertable::Insertable;
use crate::query_builder::insert_statement::InsertFromSelect;
use crate::query_builder::{Query, SelectQuery, SelectByStatement};
// use crate::query_dsl::boxed_dsl::BoxedDsl;
use crate::query_dsl::methods::*;
use crate::query_dsl::*;
// use crate::query_source::joins::{Join, JoinOn, JoinTo};
use crate::query_source::joins::JoinTo;
// use crate::query_source::QuerySource;
use crate::sql_types::{BigInt, Bool};

impl<S, STMT, Rhs, Kind, On> InternalJoinDsl<Rhs, Kind, On>
    for SelectByStatement<S, STMT>
where
    STMT: InternalJoinDsl<Rhs, Kind, On>,
{
    type Output = STMT::Output;

    fn join(self, rhs: Rhs, kind: Kind, on: On) -> Self::Output {
        self.inner.join(rhs, kind, on)
    }
}

impl<S, STMT, Selection> SelectDsl<Selection>
    for SelectByStatement<S, STMT>
where
    Selection: Expression,
    STMT: SelectDsl<Selection>,
{
    type Output = STMT::Output;

    fn select(self, selection: Selection) -> Self::Output {
        self.inner.select(selection)
    }
}

impl<ST, S, STMT> DistinctDsl for SelectByStatement<S, STMT>
where
    Self: SelectQuery<SqlType = ST>,
    STMT: DistinctDsl,
    SelectByStatement<S, STMT::Output>: SelectQuery<SqlType = ST>,
{
    type Output = SelectByStatement<S, STMT::Output>;

    fn distinct(self) -> Self::Output {
        SelectByStatement::new(self.inner.distinct())
    }
}

impl<S, STMT, Predicate> FilterDsl<Predicate>
    for SelectByStatement<S, STMT>
where
    Predicate: Expression<SqlType = Bool> + NonAggregate,
    STMT: FilterDsl<Predicate>,
{
    type Output = SelectByStatement<S, STMT::Output>;

    fn filter(self, predicate: Predicate) -> Self::Output {
        SelectByStatement::new(self.inner.filter(predicate))
    }
}

impl<S, STMT, Predicate> OrFilterDsl<Predicate>
    for SelectByStatement<S, STMT>
where
    Predicate: Expression<SqlType = Bool> + NonAggregate,
    STMT: OrFilterDsl<Predicate>,
{
    type Output = SelectByStatement<S, STMT::Output>;

    fn or_filter(self, predicate: Predicate) -> Self::Output {
        SelectByStatement::new(self.inner.or_filter(predicate))
    }
}

use crate::query_source::Table;

impl<S, STMT, PK> FindDsl<PK> for SelectByStatement<S, STMT>
where
    STMT: FindDsl<PK>,
{
    type Output = SelectByStatement<S, STMT::Output>;

    fn find(self, id: PK) -> Self::Output {
        SelectByStatement::new(self.inner.find(id))
    }
}

impl<ST, S, STMT, Expr> OrderDsl<Expr> for SelectByStatement<S, STMT>
where
    Expr: Expression,
    Self: SelectQuery<SqlType = ST>,
    STMT: OrderDsl<Expr>,
    SelectByStatement<S, STMT::Output>: SelectQuery<SqlType = ST>,
{
    type Output = SelectByStatement<S, STMT::Output>;

    fn order(self, expr: Expr) -> Self::Output {
        SelectByStatement::new(self.inner.order(expr))
    }
}

impl<ST, S, STMT, Expr> ThenOrderDsl<Expr> for SelectByStatement<S, STMT>
where
    Expr: Expression,
    Self: SelectQuery<SqlType = ST>,
    STMT: ThenOrderDsl<Expr>,
    SelectByStatement<S, STMT::Output>: SelectQuery<SqlType = ST>,
{
    type Output = SelectByStatement<S, STMT::Output>;

    fn then_order_by(self, expr: Expr) -> Self::Output {
        SelectByStatement::new(self.inner.then_order_by(expr))
    }
}

#[doc(hidden)]
pub type Limit = AsExprOf<i64, BigInt>;

impl<ST, S, STMT> LimitDsl for SelectByStatement<S, STMT>
where
    Self: SelectQuery<SqlType = ST>,
    STMT: LimitDsl,
    SelectByStatement<S, STMT::Output>: SelectQuery<SqlType = ST>,
{
    type Output = SelectByStatement<S, STMT::Output>;

    fn limit(self, limit: i64) -> Self::Output {
        SelectByStatement::new(self.inner.limit(limit))
    }
}

#[doc(hidden)]
pub type Offset = Limit;

impl<ST, S, STMT> OffsetDsl for SelectByStatement<S, STMT>
where
    Self: SelectQuery<SqlType = ST>,
    STMT: OffsetDsl,
    SelectByStatement<S, STMT::Output>: SelectQuery<SqlType = ST>,
{
    type Output = SelectByStatement<S, STMT::Output>;

    fn offset(self, offset: i64) -> Self::Output {
        SelectByStatement::new(self.inner.offset(offset))
    }
}

impl<S, STMT, Expr> GroupByDsl<Expr> for SelectByStatement<S, STMT>
where
    Self: SelectQuery,
    Expr: Expression,
    STMT: GroupByDsl<Expr>,
{
    type Output = STMT::Output;

    fn group_by(self, expr: Expr) -> Self::Output {
        self.inner.group_by(expr)
    }
}

impl<ST, S, STMT, Lock> LockingDsl<Lock> for SelectByStatement<S, STMT>
where
    Self: SelectQuery<SqlType = ST>,
    STMT: LockingDsl<Lock>,
    SelectByStatement<S, STMT::Output>: SelectQuery<SqlType = ST>,
{
    type Output = SelectByStatement<S, STMT::Output>;

    fn with_lock(self, lock: Lock) -> Self::Output {
        SelectByStatement::new(self.inner.with_lock(lock))
    }
}

impl<ST, S, STMT, Modifier> ModifyLockDsl<Modifier>
    for SelectByStatement<S, STMT>
where
    Self: SelectQuery,
    STMT: ModifyLockDsl<Modifier>,
    SelectByStatement<S, STMT::Output>: SelectQuery<SqlType = ST>,
{
    type Output = SelectByStatement<S, STMT::Output>;

    fn modify_lock(self, modifier: Modifier) -> Self::Output {
        SelectByStatement::new(self.inner.modify_lock(modifier))
    }
}

// impl<'a, F, S, D, W, O, L, Of, G, DB> BoxedDsl<'a, DB>
//     for SelectStatement<F, SelectClause<S>, D, W, O, L, Of, G>
// where
//     Self: AsQuery,
//     DB: Backend,
//     S: QueryFragment<DB> + SelectableExpression<F> + Send + 'a,
//     D: QueryFragment<DB> + Send + 'a,
//     W: Into<BoxedWhereClause<'a, DB>>,
//     O: Into<Option<Box<dyn QueryFragment<DB> + Send + 'a>>>,
//     L: QueryFragment<DB> + Send + 'a,
//     Of: QueryFragment<DB> + Send + 'a,
//     G: QueryFragment<DB> + Send + 'a,
// {
//     type Output = BoxedSelectStatement<'a, S::SqlType, F, DB>;

//     fn internal_into_boxed(self) -> Self::Output {
//         BoxedSelectStatement::new(
//             Box::new(self.select.0),
//             self.from,
//             Box::new(self.distinct),
//             self.where_clause.into(),
//             self.order.into(),
//             Box::new(self.limit),
//             Box::new(self.offset),
//             Box::new(self.group_by),
//         )
//     }
// }

// impl<'a, F, D, W, O, L, Of, G, DB> BoxedDsl<'a, DB>
//     for SelectStatement<F, DefaultSelectClause, D, W, O, L, Of, G>
// where
//     Self: AsQuery,
//     DB: Backend,
//     F: QuerySource,
//     F::DefaultSelection: QueryFragment<DB> + Send + 'a,
//     D: QueryFragment<DB> + Send + 'a,
//     W: Into<BoxedWhereClause<'a, DB>>,
//     O: Into<Option<Box<dyn QueryFragment<DB> + Send + 'a>>>,
//     L: QueryFragment<DB> + Send + 'a,
//     Of: QueryFragment<DB> + Send + 'a,
//     G: QueryFragment<DB> + Send + 'a,
// {
//     type Output = BoxedSelectStatement<'a, <F::DefaultSelection as Expression>::SqlType, F, DB>;
//     fn internal_into_boxed(self) -> Self::Output {
//         BoxedSelectStatement::new(
//             Box::new(self.from.default_selection()),
//             self.from,
//             Box::new(self.distinct),
//             self.where_clause.into(),
//             self.order.into(),
//             Box::new(self.limit),
//             Box::new(self.offset),
//             Box::new(self.group_by),
//         )
//     }
// }

impl<S, STMT> HasTable for SelectByStatement<S, STMT>
where
    STMT: HasTable,
{
    type Table = STMT::Table;

    fn table() -> Self::Table {
        STMT::table()
    }
}

// no IntoUpdateTarget: if you'd like to `.into_update_target`, you should not first `.select_by`
// impl<F, W> IntoUpdateTarget for SelectStatement<F, DefaultSelectClause, NoDistinctClause, W>
// where
//     SelectStatement<F, DefaultSelectClause, NoDistinctClause, W>: HasTable,
//     W: ValidWhereClause<F>,
// {
//     type WhereClause = W;

//     fn into_update_target(self) -> UpdateTarget<Self::Table, Self::WhereClause> {
//         UpdateTarget {
//             table: Self::table(),
//             where_clause: self.where_clause,
//         }
//     }
// }

// FIXME: Should we disable joining when `.group_by` has been called? Are there
// any other query methods where a join no longer has the same semantics as
// joining on just the table?
impl<S, STMT, Rhs> JoinTo<Rhs> for SelectByStatement<S, STMT>
where
    STMT: JoinTo<Rhs>,
{
    type FromClause = STMT::FromClause;
    type OnClause = STMT::OnClause;

    fn join_target(rhs: Rhs) -> (Self::FromClause, Self::OnClause) {
        STMT::join_target(rhs)
    }
}

impl<S, STMT> QueryDsl for SelectByStatement<S, STMT> {}

impl<S, STMT, Conn> RunQueryDsl<Conn>
    for SelectByStatement<S, STMT>
{
}

impl<S, STMT, Tab> Insertable<Tab>
    for SelectByStatement<S, STMT>
where
    Tab: Table,
    Self: Query,
{
    type Values = InsertFromSelect<Self, Tab::AllColumns>;

    fn values(self) -> Self::Values {
        InsertFromSelect::new(self)
    }
}

impl<'a, S, STMT, Tab> Insertable<Tab>
    for &'a SelectByStatement<S, STMT>
where
    Tab: Table,
    Self: Query,
{
    type Values = InsertFromSelect<Self, Tab::AllColumns>;

    fn values(self) -> Self::Values {
        InsertFromSelect::new(self)
    }
}

// impl<'a, F, S, D, W, O, L, Of, G> SelectNullableDsl
//     for SelectStatement<F, SelectClause<S>, D, W, O, L, Of, G>
// {
//     type Output = SelectStatement<F, SelectClause<Nullable<S>>, D, W, O, L, Of, G>;

//     fn nullable(self) -> Self::Output {
//         SelectStatement::new(
//             SelectClause(Nullable::new(self.select.0)),
//             self.from,
//             self.distinct,
//             self.where_clause,
//             self.order,
//             self.limit,
//             self.offset,
//             self.group_by,
//             self.locking,
//         )
//     }
// }

// impl<'a, F, D, W, O, L, Of, G> SelectNullableDsl
//     for SelectStatement<F, DefaultSelectClause, D, W, O, L, Of, G>
// where
//     F: QuerySource,
// {
//     type Output =
//         SelectStatement<F, SelectClause<Nullable<F::DefaultSelection>>, D, W, O, L, Of, G>;

//     fn nullable(self) -> Self::Output {
//         SelectStatement::new(
//             SelectClause(Nullable::new(self.from.default_selection())),
//             self.from,
//             self.distinct,
//             self.where_clause,
//             self.order,
//             self.limit,
//             self.offset,
//             self.group_by,
//             self.locking,
//         )
//     }
// }
