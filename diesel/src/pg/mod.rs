//! Provides types and functions related to working with PostgreSQL
//!
//! Much of this module is re-exported from database agnostic locations.
//! However, if you are writing code specifically to extend Diesel on
//! PostgreSQL, you may need to work with this module directly.

pub mod expression;
pub mod types;
pub mod upsert;

mod backend;
#[cfg(feature = "postgres")]
mod connection;
mod metadata_lookup;
#[cfg(feature = "unstable_pure_rust_postgres")]
mod postgres_connection;
mod query_builder;
pub(crate) mod serialize;
mod transaction;
mod value;

pub use self::backend::{Pg, PgTypeMetadata};
#[cfg(feature = "postgres")]
pub use self::connection::PgConnection;
pub use self::metadata_lookup::PgMetadataLookup;
#[cfg(feature = "unstable_pure_rust_postgres")]
pub use self::postgres_connection::PostgresConnection;
pub use self::query_builder::DistinctOnClause;
pub use self::query_builder::PgQueryBuilder;
pub use self::transaction::TransactionBuilder;
pub use self::value::PgValue;

/// Data structures for PG types which have no corresponding Rust type
///
/// Most of these types are used to implement `ToSql` and `FromSql` for higher
/// level types.
pub mod data_types {
    #[doc(inline)]
    pub use super::types::date_and_time::{PgDate, PgInterval, PgTime, PgTimestamp};
    #[doc(inline)]
    pub use super::types::floats::PgNumeric;
    #[doc(inline)]
    pub use super::types::money::PgMoney;
    pub use super::types::money::PgMoney as Cents;
}
