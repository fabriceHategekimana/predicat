pub mod base_parser;

pub use base_parser::*;

fn parse_get(s: &str) -> IResult<&str,Language> {
    let res = tag("get")(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Get)),
        Err(e) => Err(e)
    }
}

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
