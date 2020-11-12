#[macro_use]
extern crate diesel;

table! {
    users {
        id -> Integer,
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = users)]
struct UserStruct {
    name: String,
    #[diesel(column_name = hair_color)]
    color_de_pelo: String,
}

#[derive(AsChangeset)]
#[diesel(table_name = users)]
struct UserTuple(#[diesel(column_name = name)] String);

fn main() {}
