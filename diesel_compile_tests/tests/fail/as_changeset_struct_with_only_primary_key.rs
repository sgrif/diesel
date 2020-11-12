#[macro_use]
extern crate diesel;

table! {
    foo {
        id -> Integer,
        bar -> Integer,
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = foo)]
struct Foo1 {
    id: i32,
    bar: i32,
}

#[derive(AsChangeset)]
#[diesel(table_name = foo)]
struct Foo2 {
    id: i32,
}

fn main() {}
