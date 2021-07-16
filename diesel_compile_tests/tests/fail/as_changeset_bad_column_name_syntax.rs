#[macro_use]
extern crate diesel;

table! {
    users {
        id -> Integer,
        name -> Text,
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = users)]
struct User {
    #[diesel(column_name)]
    name: String,
}

fn main() {}
