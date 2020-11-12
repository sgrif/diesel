#[macro_use]
extern crate diesel;

table! {
    users {
        id -> Integer,
    }
}

#[derive(Insertable)]
#[diesel(table_name = users)]
struct UserOk {
    id: i32,
}

#[derive(Insertable)]
#[diesel(table_name(users))]
struct UserWarn {
    id: i32,
}

#[derive(Insertable)]
#[diesel(table_name)]
struct UserError1 {
    id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = true)]
struct UserError2 {
    id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = "")]
struct UserError3 {
    id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = "not a path")]
struct UserError4 {
    id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = does::not::exist)]
struct UserError5 {
    id: i32,
}

fn main() {}
