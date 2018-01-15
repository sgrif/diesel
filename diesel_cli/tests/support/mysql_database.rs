use diesel::connection::SimpleConnection;
use diesel::dsl::sql;
use diesel::sql_types::Bool;
use diesel::*;

pub struct Database {
    url: String,
}

impl Database {
    pub fn new(url: &str) -> Self {
        Database { url: url.into() }
    }

    pub fn create(self) -> Self {
        let (database, mysql_url) = self.split_url();
        let conn = MysqlConnection::establish(&mysql_url).unwrap();
        conn.execute(&format!("CREATE DATABASE `{}`", database))
            .unwrap();
        self
    }

    pub fn exists(&self) -> bool {
        MysqlConnection::establish(&self.url).is_ok()
    }

    pub fn table_exists(&self, table: &str) -> bool {
        select(sql::<Bool>(&format!(
            "EXISTS \
                (SELECT 1 \
                 FROM information_schema.tables \
                 WHERE table_name = '{}'
                 AND table_schema = DATABASE())",
            table
        ))).get_result(&self.conn())
            .unwrap()
    }

    pub fn conn(&self) -> MysqlConnection {
        MysqlConnection::establish(&self.url)
            .expect(&format!("Failed to open connection to {}", &self.url))
    }

    pub fn execute(&self, command: &str) {
        self.conn()
            .batch_execute(command)
            .expect(&format!("Error executing command {}", command));
    }

    fn split_url(&self) -> (String, String) {
        let mut split: Vec<&str> = self.url.split("/").collect();
        let default_database = "information_schema";
        let database_name_with_arguments: Vec<&str> = split.pop().unwrap().split('?').collect();
        let database = database_name_with_arguments[0];
        let mysql_url;
        match database_name_with_arguments.len() {
            2 => {
                let args : &str = database_name_with_arguments[1];
                mysql_url = format!("{}/{}?{}", split.join("/"), default_database, args);
            },
            _ => mysql_url = format!("{}/{}", split.join("/"), default_database)
        }
        (database.into(), mysql_url)
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        let (database, mysql_url) = self.split_url();
        let conn = try_drop!(
            MysqlConnection::establish(&mysql_url),
            "Couldn't connect to database"
        );
        try_drop!(
            conn.execute(&format!("DROP DATABASE IF EXISTS `{}`", database)),
            "Couldn't drop database"
        );
    }
}
