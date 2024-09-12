use crate::var::Var;

fn with_dollard(a: &str) -> String {
    format!("${}", a)
}

#[derive(Clone, PartialEq, Debug)]
pub enum Triplet {
    Teee(String, String, String),
    Tvee(String, String, String),
    Teve(String, String, String),
    Teev(String, String, String),
    Tvve(String, String, String),
    Tvev(String, String, String),
    Tevv(String, String, String),
    Tvvv(String, String, String),
    TNeee(String, String, String),
    TNvee(String, String, String),
    TNeve(String, String, String),
    TNeev(String, String, String),
    TNvve(String, String, String),
    TNvev(String, String, String),
    TNevv(String, String, String),
    TNvvv(String, String, String),
    Empty
}

impl Triplet {
    pub fn invert(self) -> Triplet {
        match self {
            Triplet::Teee(a,b,c) => Triplet::Teee(a,b,c),
            Triplet::Tvee(a,b,c) => Triplet::Tvee(a,b,c),
            Triplet::Teve(a,b,c) => Triplet::Teve(a,b,c),
            Triplet::Teev(a,b,c) => Triplet::Teev(a,b,c),
            Triplet::Tvve(a,b,c) => Triplet::Tvve(a,b,c),
            Triplet::Tvev(a,b,c) => Triplet::Tvev(a,b,c),
            Triplet::Tevv(a,b,c) => Triplet::Tevv(a,b,c),
            Triplet::Tvvv(a,b,c) => Triplet::Tvvv(a,b,c),
            Triplet::TNeee(a,b,c) => Triplet::Teee(a,b,c),
            Triplet::TNvee(a,b,c) => Triplet::Tvee(a,b,c),
            Triplet::TNeve(a,b,c) => Triplet::Teve(a,b,c),
            Triplet::TNeev(a,b,c) => Triplet::Teev(a,b,c),
            Triplet::TNvve(a,b,c) => Triplet::Tvve(a,b,c),
            Triplet::TNvev(a,b,c) => Triplet::Tvev(a,b,c),
            Triplet::TNevv(a,b,c) => Triplet::Tevv(a,b,c),
            Triplet::TNvvv(a,b,c) => Triplet::Tvvv(a,b,c),
            rest => rest
        }
    }

    pub fn to_tuple(&self) -> (String, String, String) {
        match self {
            Triplet::Teee(a,b,c) => (a.to_string(),b.to_string(),c.to_string()),
            Triplet::Tvee(a,b,c) => (with_dollard(a),b.to_string(),c.to_string()),
            Triplet::Teve(a,b,c) => (a.to_string(),with_dollard(b),c.to_string()),
            Triplet::Teev(a,b,c) => (a.to_string(),b.to_string(),with_dollard(c)),
            Triplet::Tvve(a,b,c) => (with_dollard(a),with_dollard(b),c.to_string()),
            Triplet::Tvev(a,b,c) => (with_dollard(a),b.to_string(),with_dollard(c)),
            Triplet::Tevv(a,b,c) => (a.to_string(),with_dollard(b),with_dollard(c)),
            Triplet::Tvvv(a,b,c) => (with_dollard(a),with_dollard(b),with_dollard(c)),
            Triplet::Empty => ("".to_string(), "".to_string(), "".to_string()),
            tri => tri.clone().invert().to_tuple()
        }
    }
    pub fn to_tuple_with_variable(&self) -> (String, String, String) {
        match self {
            Triplet::Teee(a,b,c) => (a.to_string(),b.to_string(),c.to_string()),
            Triplet::Tvee(a,b,c) => (Var::format(&a),b.to_string(),c.to_string()),
            Triplet::Teve(a,b,c) => (a.to_string(),Var::format(&b),c.to_string()),
            Triplet::Teev(a,b,c) => (a.to_string(),b.to_string(), Var::format(&c)),
            Triplet::Tvve(a,b,c) => (Var::format(&a),Var::format(&b),c.to_string()),
            Triplet::Tvev(a,b,c) => (Var::format(&a),b.to_string(),Var::format(&c)),
            Triplet::Tevv(a,b,c) => (a.to_string(),Var::format(&b),Var::format(&c)),
            Triplet::Tvvv(a,b,c) => (Var::format(&a),Var::format(&b),Var::format(&c)),
            Triplet::Empty => ("".to_string(), "".to_string(), "".to_string()),
            tri => tri.clone().invert().to_tuple_with_variable()
        }
    }

    pub fn display(&self) -> String {
        match self {
            Triplet::Teee(a,b,c) => format!("{},{},{}", a.to_string(),b.to_string(),c.to_string()),
            Triplet::Tvee(a,b,c) => format!("{},{},{}", Var::format(&a),b.to_string(),c.to_string()),
            Triplet::Teve(a,b,c) => format!("{},{},{}", a.to_string(),Var::format(&b),c.to_string()),
            Triplet::Teev(a,b,c) => format!("{},{},{}", a.to_string(),b.to_string(), Var::format(&c)),
            Triplet::Tvve(a,b,c) => format!("{},{},{}", Var::format(&a),Var::format(&b),c.to_string()),
            Triplet::Tvev(a,b,c) => format!("{},{},{}", Var::format(&a),b.to_string(),Var::format(&c)),
            Triplet::Tevv(a,b,c) => format!("{},{},{}", a.to_string(),Var::format(&b),Var::format(&c)),
            Triplet::Tvvv(a,b,c) => format!("{},{},{}", Var::format(&a),Var::format(&b),Var::format(&c)),
            Triplet::TNeee(a,b,c) => format!("Not({},{},{})", a.to_string(),b.to_string(),c.to_string()),
            Triplet::TNvee(a,b,c) => format!("Not({},{},{})", Var::format(&a),b.to_string(),c.to_string()),
            Triplet::TNeve(a,b,c) => format!("Not({},{},{})", a.to_string(),Var::format(&b),c.to_string()),
            Triplet::TNeev(a,b,c) => format!("Not({},{},{})", a.to_string(),b.to_string(), Var::format(&c)),
            Triplet::TNvve(a,b,c) => format!("Not({},{},{})", Var::format(&a),Var::format(&b),c.to_string()),
            Triplet::TNvev(a,b,c) => format!("Not({},{},{})", Var::format(&a),b.to_string(),Var::format(&c)),
            Triplet::TNevv(a,b,c) => format!("Not({},{},{})", a.to_string(),Var::format(&b),Var::format(&c)),
            Triplet::TNvvv(a,b,c) => format!("Not({},{},{})", Var::format(&a),Var::format(&b),Var::format(&c)),
            Triplet::Empty => "Empty triplet".to_string()
        }
    }
}


impl From<Triplet> for String {
    fn from(t: Triplet) -> String {
        let tri : (String, String, String) = t.into();
        format!("{} {} {}", tri.0, tri.1, tri.2)
    }
}

impl From<Triplet> for (String, String, String) {
   fn from(val: Triplet) -> Self {
        match val {
        Triplet::Teee(a,b,c) => (a,b,c),
        Triplet::Tvee(a,b,c) => (a,b,c),
        Triplet::Teve(a,b,c) => (a,b,c),
        Triplet::Teev(a,b,c) => (a,b,c),
        Triplet::Tvve(a,b,c) => (a,b,c),
        Triplet::Tvev(a,b,c) => (a,b,c),
        Triplet::Tevv(a,b,c) => (a,b,c),
        Triplet::Tvvv(a,b,c) => (a,b,c),
        Triplet::Empty => ("".to_string(), "".to_string(), "".to_string()),
        tri => tri.clone().invert().to_tuple()
    }
   } 
}
