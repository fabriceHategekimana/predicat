mod parser;
mod importer;
mod knowledge;

// ajout d'un module parseur

use crate::parser::parse_query;
use crate::knowledge::initialisation;

fn main() {
    //let res = parse_query("get $A $B such_as $A grade $B and $B == 'sgt' ");
    //let res = parse_query("get $A such_as $A grade sgt ");
    //let res = parse_query("get $A such_as $B type $A ");
    let res = parse_query("get $A such_as $A == 'sgt' ");
    println!("res: {:?}", res);
    if &res[0..6] == "Select" {
        initialisation(&res);
    }
}

