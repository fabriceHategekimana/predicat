pub use crate::parser::base_parser::{
    Language,
    Triplet,
    Triplet::*,
    Comp,
    Var,
    Tri,
    IResult,
    preceded,
    tag,
    space1,
    alt,
    recognize,
    char,
    alphanumeric1,
    delimited,
    is_not,
    tuple,
    digit1,
    terminated,
    many1,
    parse_variable,
    parse_triplet_and,
    Error,
    ErrorKind,
    triplet_to_sql
};


fn format_comparisons(comp: &[Language]) -> String {
    if  comp == [Language::Empty] {
        String::from(";")
    }
    else {
        let comparisons = comp.iter()
            .filter_map(|x| {
                match x {
                    Comp(c) => Some(c.replace("$","").replace("==","=")),
                    _ => None
                }
            });
        let final_comparisons = comparisons
            .reduce(|acc, x| format!("{} AND{}", acc, x)).unwrap();
        format!(" WHERE{};", final_comparisons)
    }
}

fn format_variables(vars: &[Language]) -> String {
    if vars == [Language::Empty]{
        String::from("SELECT * FROM ")
    }
    else {
        let extracted_vars = vars.iter()
            .filter_map(|x| {
                match x {
                    Var(v) => Some(v),
                    _ => None
                }
            });
        let string_vars = extracted_vars
            .fold("".to_string(), |acc, &x| acc +","+x)
            .chars()
            .skip(1)
            .collect::<String>();
        format!("SELECT {} FROM ",string_vars)
    }
}

fn format_triplets(tri: &[Language]) -> String {
    if tri == [Language::Empty]{
        String::from("facts")
    }
    else {
        let sql_queries = tri.iter()
            .filter_map(|x| {
                match x {
                    Tri(t) => Some(triplet_to_sql(&t)),
                    _ => None
                }
            });
        let queries = sql_queries
            .reduce(|acc, x| format!("{} natural join {}", acc, x)).unwrap();
        format!("({})", queries)
    }
}

fn to_sql(res: (&[Language], &[Language], &[Language])) -> String {
    let head = format_variables(&res.0);
    let columns = format_triplets(&res.1); // warning, put the result into a parenthese
    let comparisons = format_comparisons(&res.2);
    format!("{}{}{}", head, columns, comparisons )
}

fn parse_operator(s: &str) -> IResult<&str,&str> {
    preceded(space1, alt((
        tag("=="),
        tag("!="),
        tag("<="),
        tag(">="),
        tag("<"),
        tag(">")
        )))(s)
}

fn recognize_variable(s: &str) -> IResult<&str,&str> {
    preceded(
        space1,
        recognize(preceded(char('$'), alphanumeric1)),
        )(s)
}

fn parse_string(s: &str) -> IResult<&str,&str> {
   recognize(preceded(space1,delimited(
       char('\''),
       is_not("\'"),
       char('\''))))(s)
}

fn parse_number(s: &str) -> IResult<&str,&str> {
   preceded(space1,
      alt((
          recognize(tuple((char('-'),digit1,char('.'),digit1))),
          recognize(tuple((digit1,char('.'),digit1))),
          recognize(tuple((char('-'), digit1))),
          digit1)
       ))(s)
}

fn parse_value(s: &str) -> IResult<&str,&str> {
    alt((parse_string, parse_number))(s)
}

fn parse_valvar(s: &str) -> IResult<&str,&str> {
    alt((recognize_variable, parse_value))(s)
}


fn parse_comparison_and(s: &str) -> IResult<&str,Language> {
    alt((
        terminated(parse_comparison, tag(" and")),
        parse_comparison))(s)
}

fn parse_comparison(s: &str) -> IResult<&str,Language> {
    let res = recognize(tuple((
                parse_valvar,
                parse_operator,
                parse_valvar
                )))(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Comp(s))),
        Err(e) => Err(e)
    }
}



fn parse_connector(s: &str) -> IResult<&str, Language> {
    let res =alt((tag(" such_as"),
        tag(" who_is"),
        tag(" who_are"),
        tag(" who_has")))(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Connector)),
        Err(e) => Err(e)
    }
}

fn parse_get(s: &str) -> IResult<&str,Language> {
    let res = tag("get")(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Get)),
        Err(e) => Err(e)
    }
}


// get [vars] [connector] [triplets] [comparison]
fn parse_query_var1(s: &str) -> IResult<&str,(Vec<Language>, Vec<Language>,Vec<Language>)> {
    let res = tuple((parse_get,
          many1(parse_variable),
          parse_connector,
          many1(parse_triplet_and),
          many1(parse_comparison_and)))(s);
    match res {
        Ok((r, (g,var,c,tri,comp))) => Ok((r, (var, tri, comp))),
        Err(e) => Err(e)
    }
}

// get [variables] [connector] [triplets]
fn parse_query_var2(s: &str) -> IResult<&str,(Vec<Language>, Vec<Language>,Vec<Language>)> {
    let res = tuple((parse_get,
          many1(parse_variable),
          parse_connector,
          many1(parse_triplet_and)))(s);
    match res {
        Ok((r, (g,var,c,tri))) => Ok((r, (var, tri, vec![Language::Empty]))),
        Err(e) => Err(e)
    }
}

// get [vars] [connector] [comparison]
fn parse_query_var3(s: &str) -> IResult<&str,(Vec<Language>, Vec<Language>,Vec<Language>)> {
    let res = tuple((parse_get,
          many1(parse_variable),
          parse_connector,
          many1(parse_comparison_and)))(s);
    match res {
        Ok((r, (g,var,c,comp))) => Ok((r, (var, vec![Language::Empty], comp))),
        Err(e) => Err(e)
    }
}

//return a vector to correlate with the result of the modifiers
pub fn parse_query(s: &str) -> IResult<&str,Vec<String>> {
    let res = alt((
        parse_query_var1,
        parse_query_var2,
        parse_query_var3
        ))(s);
    match res {
        Ok((t, (v1,v2,v3))) => Ok((t, vec![to_sql((&v1,&v2,&v3))])),
        Err(e) => Err(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        assert_eq!(parse_get("get").unwrap().1, Language::Get);
        assert_eq!(
            parse_get("SELECT"),
            Err(nom::Err::Error(
                    Error {
                        input: "SELECT",
                        code: ErrorKind::Tag})));
    }

    #[test]
    fn test_variable() {
        assert_eq!(
            parse_variable(" $Hello").unwrap().1,
            Language::Var("Hello"));
        assert_eq!(
            parse_variable(" $A").unwrap().1,
            Language::Var("A"));
        assert_eq!(
            parse_variable(" $2").unwrap().1,
            Language::Var("2"));
        assert_eq!(
            parse_variable(" hey"),
            Err(nom::Err::Error(
                Error {
                    input: "hey",
                    code: ErrorKind::Char
                }
            )));
    } 
    #[test]
    fn test_connector() {
        assert_eq!(
            parse_connector(" who_has").unwrap().1,
            Language::Connector);
        assert_eq!(
            parse_connector(" such_as").unwrap().1,
            Language::Connector);
        assert_eq!(
            parse_connector(" hey"),
            Err(nom::Err::Error(
                Error {
                    input: " hey",
                    code: ErrorKind::Tag
                }
            )));
    }



    #[test]
    fn test_operator() {
        assert_eq!(
            parse_operator(" ==").unwrap().1,
            "==");
        assert_eq!(
            parse_operator(" >").unwrap().1,
            ">");
        assert_eq!(
            parse_operator(" a"),
            Err(nom::Err::Error(
                Error {
                    input: "a",
                    code: ErrorKind::Tag
                }
            )));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(
            parse_comparison(" 4 < 5").unwrap().1,
            Language::Comp(" 4 < 5"));
        assert_eq!(
            parse_comparison(" $A > 5").unwrap().1,
            Language::Comp(" $A > 5"));
        assert_eq!(
            parse_comparison(" F"),
            Err(
                nom::Err::Error(
                    Error { input: "F", code: ErrorKind::Digit })));
        assert_eq!(
            parse_comparison(" 4 == 5").unwrap().1,
            Language::Comp(" 4 == 5"));
        assert_eq!(
            parse_comparison(" 4 == 'res'").unwrap().1,
            Language::Comp(" 4 == 'res'"));
    }

    #[test]
    fn test_string() {
        assert_eq!(
            parse_string(" 'un deux trois'").unwrap().1,
            " 'un deux trois'"
            );
        assert_eq!(
            parse_string(" 'sdt'").unwrap().1,
            " 'sdt'"
            );
    }

    #[test]
    fn test_number() {
        assert_eq!(
            parse_number(" 57").unwrap().1,
            "57");
        assert_eq!(
            parse_number(" 2.57").unwrap().1,
            "2.57");
        assert_eq!(
            parse_number(" -57").unwrap().1,
            "-57");
        assert_eq!(
            parse_number(" -57.34").unwrap().1,
            "-57.34");
    }

    #[test]
    fn test_valvar() {
        assert_eq!(
            parse_valvar(" $A").unwrap().1,
            "$A");
        assert_eq!(
            parse_valvar(" 7").unwrap().1,
            "7");
        assert_eq!(
            parse_valvar(" '7'").unwrap().1,
            " '7'");
    }

    #[test]
    fn test_comparison_and() {
        assert_eq!(
            parse_comparison_and(" 7 == 8 AND 6 < 9").unwrap().1,
            Comp(" 7 == 8"));
    }
    #[test]
    fn test_format_variables() {
        assert_eq!(
            format_variables(&vec![Var("X"),Var("Y")]),
            "SELECT X,Y FROM "
        );
        assert_eq!(
            format_variables(&vec![Var("X")]),
            "SELECT X FROM "
        );
    }

    #[test]
    fn test_format_triplets() {
        assert_eq!(
            format_triplets(&vec![Tri(Tvvv("A","B","C"))]),
            "(SELECT subject AS A,link AS B,goal AS C FROM facts)".to_string()
        );
        assert_eq!(
            format_triplets(&vec![Tri(Tvvv("A","B","C")),Tri(Twvv("D","E","F"))]),
            "(SELECT subject AS A,link AS B,goal AS C FROM facts natural join SELECT link AS E,goal AS F FROM facts WHERE subject='D')".to_string()
        );
    }

    #[test]
    fn test_format_comparisons() {
        assert_eq!(
            format_comparisons(&vec![Comp(" $A == 8")]),
            " WHERE A = 8;".to_string()
        );
        assert_eq!(
            format_comparisons(&vec![Comp(" $A == 8"), Comp(" 6 < 3")]),
            " WHERE A = 8 AND 6 < 3;".to_string()
        );
    }

    #[test]
    fn test_value() {
        assert_eq!(
            parse_value(" -57.34").unwrap().1,
            "-57.34");

        assert_eq!(
            parse_value(" '3'").unwrap().1,
            " '3'"
            );

        assert_eq!(
            parse_value(" 'sdt'").unwrap().1,
            " 'sdt'"
            );
    }
}
