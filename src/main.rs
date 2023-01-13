mod parser;
mod importer;

// ajout d'un module parseur

use crate::parser::parse_query;

fn main() {
    let res = parse_query("get $A $B $C such_as $A ami $B and $B == $C ");
    println!("res: {:?}", res);
}

