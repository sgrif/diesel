extern crate diesel;

use diesel::sql_types::SqlType;

#[derive(SqlType)]
#[diesel(postgres_type)]
struct Type1;

#[derive(SqlType)]
#[diesel(postgres_type(type_name = "foo", oid = 2, array_oid = 3))]
struct Type2;

#[derive(SqlType)]
#[diesel(postgres_type(oid = 2))]
struct Type3;

#[derive(SqlType)]
#[diesel(postgres_type(oid = 1, array_oid = "1"))]
struct Type4;

#[derive(SqlType)]
#[diesel(postgres_type(oid = 1, ary_oid = "1"))]
struct Type5;

#[derive(SqlType)]
#[diesel(postgres_type = "foo")]
struct Type6;

fn main() {}
