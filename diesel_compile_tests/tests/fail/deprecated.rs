#[macro_use]
extern crate diesel;

table! {
    users {
        id -> Integer,
        name -> Text,
    }
}

#[derive(Identifiable)]
#[table_name = "users"]
#[primary_key(id)]
struct UserForm {
    id: i32,
    #[column_name = "name"]
    name: String,
}

fn main() {
    // Workaround for https://github.com/dtolnay/trybuild/issues/8
    compile_error!("warnings");
}
