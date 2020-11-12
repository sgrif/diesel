#[macro_use]
extern crate diesel;

table! {
    users {
        id -> Integer,
    }
}

#[derive(Insertable)]
struct Post {
    id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = posts)]
struct Post2 {
    id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
struct User1 {
    name: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
struct User2 {
    #[diesel(column_name = name)]
    name: String,
}

fn main() {}
