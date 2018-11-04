#[cfg(feature = "bigdecimal")]
pub mod bigdecimal {
    extern crate bigdecimal;

    use self::bigdecimal::BigDecimal;
    use std::io::prelude::*;

    use deserialize::{self, FromSql};
    use mysql_like::MysqlLikeBackend;
    use serialize::{self, IsNull, Output, ToSql};
    use sql_types::{Binary, Numeric};

    impl<MysqlLike: MysqlLikeBackend> ToSql<Numeric, MysqlLike> for BigDecimal {
        fn to_sql<W: Write>(&self, out: &mut Output<W, MysqlLike>) -> serialize::Result {
            write!(out, "{}", *self)
                .map(|_| IsNull::No)
                .map_err(|e| e.into())
        }
    }

    impl<MysqlLike> FromSql<Numeric, MysqlLike> for BigDecimal
        where MysqlLike: MysqlLikeBackend {
        fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
            let bytes_ptr = <*const [u8] as FromSql<Binary, MysqlLike>>::from_sql(bytes)?;
            let bytes = unsafe { &*bytes_ptr };
            BigDecimal::parse_bytes(bytes, 10)
                .ok_or_else(|| Box::from(format!("{:?} is not valid decimal number ", bytes)))
        }
    }
}
