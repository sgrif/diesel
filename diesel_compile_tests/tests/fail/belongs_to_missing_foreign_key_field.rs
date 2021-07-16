#[macro_use]
extern crate diesel;

struct Bar;

#[derive(Associations)]
#[diesel(belongs_to(Bar))]
#[diesel(belongs_to(Bar, foreign_key = bar_id))]
struct Foo {}

#[derive(Associations)]
#[diesel(belongs_to(Bar))]
#[diesel(belongs_to(Bar, foreign_key = bar_id))]
struct Baz {
    #[diesel(column_name = baz_id)]
    bar_id: i32,
}

fn main() {}
