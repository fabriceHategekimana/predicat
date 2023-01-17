mod parser;
mod importer;
mod knowledge;

// ajout d'un module parseur

use crate::parser::parse_query;
use crate::knowledge::initialisation;

fn main() {
    let res = parse_query("get $A $B $C such_as $A ami $B and $B == $C ");
    //println!("res: {:?}", res);
    initialisation();
}

