use diesel::*;
use schema::TestBackend;

#[test]
fn test_debug_count_output() {
    use schema::users::dsl::*;
    let sql = debug_query::<TestBackend, _>(&users.count()).to_string();
    if cfg!(any(feature = "postgres", feature = "postgres_pure_rust")) {
        assert_eq!(sql, r#"SELECT COUNT(*) FROM "users" -- binds: []"#);
    } else {
        assert_eq!(sql, "SELECT COUNT(*) FROM `users` -- binds: []");
    }
}

#[test]
fn test_debug_output() {
    use schema::users::dsl::*;
    let command = update(users.filter(id.eq(1))).set(name.eq("new_name"));
    let sql = debug_query::<TestBackend, _>(&command).to_string();
    if cfg!(any(feature = "postgres", feature = "postgres_pure_rust")) {
        assert_eq!(
            sql,
            r#"UPDATE "users" SET "name" = $1 WHERE "users"."id" = $2 -- binds: ["new_name", 1]"#
        )
    } else {
        assert_eq!(
            sql,
            r#"UPDATE `users` SET `name` = ? WHERE `users`.`id` = ? -- binds: ["new_name", 1]"#
        )
    }
}

#[test]
fn test_debug_batch_insert() {
    // This test ensures that we've implemented `debug_query` for batch insert
    // on sqlite
    // This requires a separate impl because it's more than one sql statement that
    // is executed

    use schema::users::dsl::*;

    let values = vec![
        (name.eq("Sean"), hair_color.eq(Some("black"))),
        (name.eq("Tess"), hair_color.eq(None::<&str>)),
    ];
    let borrowed_command = insert_into(users).values(&values);
    let borrowed_sql_display = debug_query::<TestBackend, _>(&borrowed_command).to_string();
    let borrowed_sql_debug = format!("{:?}", debug_query::<TestBackend, _>(&borrowed_command));

    let owned_command = insert_into(users).values(values);
    let owned_sql_display = debug_query::<TestBackend, _>(&owned_command).to_string();
    let owned_sql_debug = format!("{:?}", debug_query::<TestBackend, _>(&owned_command));

    if cfg!(feature = "postgres") {
        assert_eq!(
            borrowed_sql_display,
            r#"INSERT INTO "users" ("name", "hair_color") VALUES ($1, $2), ($3, $4) -- binds: ["Sean", Some("black"), "Tess", None]"#
        );
        assert_eq!(
            borrowed_sql_debug,
            r#"Query { sql: "INSERT INTO \"users\" (\"name\", \"hair_color\") VALUES ($1, $2), ($3, $4)", binds: ["Sean", Some("black"), "Tess", None] }"#
        );

        assert_eq!(
            owned_sql_display,
            r#"INSERT INTO "users" ("name", "hair_color") VALUES ($1, $2), ($3, $4) -- binds: ["Sean", Some("black"), "Tess", None]"#
        );
        assert_eq!(
            owned_sql_debug,
            r#"Query { sql: "INSERT INTO \"users\" (\"name\", \"hair_color\") VALUES ($1, $2), ($3, $4)", binds: ["Sean", Some("black"), "Tess", None] }"#
        );
    } else if cfg!(feature = "sqlite") {
        assert_eq!(
            borrowed_sql_display,
            r#"BEGIN;
INSERT INTO `users` (`name`, `hair_color`) VALUES (?, ?) -- binds: ["Sean", Some("black")]
INSERT INTO `users` (`name`, `hair_color`) VALUES (?, ?) -- binds: ["Tess", None]
COMMIT;
"#
        );
        assert_eq!(
            borrowed_sql_debug,
            r#"Query { sql: ["BEGIN", "INSERT INTO `users` (`name`, `hair_color`) VALUES (?, ?) -- binds: [\"Sean\", Some(\"black\")]", "INSERT INTO `users` (`name`, `hair_color`) VALUES (?, ?) -- binds: [\"Tess\", None]", "COMMIT"], binds: [] }"#
        );

        assert_eq!(
            owned_sql_display,
            r#"BEGIN;
INSERT INTO `users` (`name`, `hair_color`) VALUES (?, ?) -- binds: ["Sean", Some("black")]
INSERT INTO `users` (`name`, `hair_color`) VALUES (?, ?) -- binds: ["Tess", None]
COMMIT;
"#
        );
        assert_eq!(
            owned_sql_debug,
            r#"Query { sql: ["BEGIN", "INSERT INTO `users` (`name`, `hair_color`) VALUES (?, ?) -- binds: [\"Sean\", Some(\"black\")]", "INSERT INTO `users` (`name`, `hair_color`) VALUES (?, ?) -- binds: [\"Tess\", None]", "COMMIT"], binds: [] }"#
        );
    } else {
        assert_eq!(
            borrowed_sql_display,
            r#"INSERT INTO `users` (`name`, `hair_color`) VALUES (?, ?), (?, ?) -- binds: ["Sean", Some("black"), "Tess", None]"#
        );
        assert_eq!(
            borrowed_sql_debug,
            r#"Query { sql: "INSERT INTO `users` (`name`, `hair_color`) VALUES (?, ?), (?, ?)", binds: ["Sean", Some("black"), "Tess", None] }"#
        );

        assert_eq!(
            owned_sql_display,
            r#"INSERT INTO `users` (`name`, `hair_color`) VALUES (?, ?), (?, ?) -- binds: ["Sean", Some("black"), "Tess", None]"#
        );
        assert_eq!(
            owned_sql_debug,
            r#"Query { sql: "INSERT INTO `users` (`name`, `hair_color`) VALUES (?, ?), (?, ?)", binds: ["Sean", Some("black"), "Tess", None] }"#
        );
    }
}
