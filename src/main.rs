mod parser;
mod importer;

// ajout d'un module parseur

use crate::parser::parse_query;
use sqlite::Connection;

fn get(connection: Connection) {
    let query = "SELECT * FROM users WHERE age > 50";
    connection
        .iterate(query, |pairs| {
            for &(name, value) in pairs.iter() {
                println!("{} = {}", name, value.unwrap());
            }
            true
        })
        .unwrap();
}

fn main() {
    let res = parse_query("get $A $B $C such_as $A ami $B and $B == $C ");
    println!("res: {:?}", res);

    let connection = sqlite::open("data.db").unwrap();

    let query = "
        CREATE TABLE users (name TEXT, age INTEGER);
        INSERT INTO users VALUES ('Alice', 42);
        INSERT INTO users VALUES ('Bob', 69);
    ";
    connection.execute(query).unwrap();
    let res = get(connection);
    println!("res: {:?}", res);
}

