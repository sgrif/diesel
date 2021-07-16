#[macro_use]
extern crate diesel;

table! {
    bar (id) {
        id -> Integer,
    }
}

table! {
    foo (bar_id) {
        bar_id -> Integer,
    }
}

#[derive(Identifiable)]
#[diesel(table_name = bar)]
struct Bar {
    id: i32,
}

#[derive(Identifiable)]
#[diesel(table_name = bar)]
struct Baz {
    id: i32,
}

#[derive(Associations)]
#[diesel(belongs_to)]
#[diesel(belongs_to = "Bar")]
#[diesel(belongs_to())]
#[diesel(belongs_to(foreign_key = bar_id))]
#[diesel(belongs_to(Bar, foreign_key))]
#[diesel(belongs_to(Bar, foreign_key(bar_id)))]
#[diesel(belongs_to(Baz, foreign_key = bar_id, random_option))]
#[diesel(table_name = foo)]
struct Foo {
    bar_id: i32,
}

fn main() {}
