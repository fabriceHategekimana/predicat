pub mod base_parser;

pub use base_parser::*;

fn triplet_to_delete(tri: &Triplet) -> String {
    let tup = tri.to_tuple_with_variable();
    format!("DELETE FROM facts WHERE subject='{}',link='{}',goal='{}'",
            tup.0, tup.1, tup.2)
}

fn delete_to_insert(l: &Language) -> String {
    match l {
        Language::Tri(tri) => triplet_to_delete(&tri),
        _ => String::from("")
    }
}

fn parse_delete_modifier(s: &str) -> IResult<&str,Vec<String>> {
    let res = preceded(tag("delete"),
        many1(parse_triplet_and)
    )(s);
    match res {
        Ok((s, v)) => Ok((s, v.iter().map(|x| delete_to_insert(x)).collect::<Vec<String>>())),
        Err(e) => Err(e)
    }
}

fn triplet_to_insert(tri: &Triplet) -> String {
    let tup = tri.to_tuple_with_variable();
    format!("INSERT INTO facts (subject,link,goal) VALUES ({},{},{})",
            tup.0, tup.1, tup.2)
}

fn add_to_insert(l: &Language) -> String {
    match l {
        Language::Tri(tri) => triplet_to_insert(&tri),
        _ => String::from("")
    }
}

fn parse_add_modifier(s: &str) -> IResult<&str, Vec<String>> {
    let res = preceded(tag("add"),
        many1(parse_triplet_and)
    )(s);
    match res {
        Ok((s, v)) => Ok((s, v.iter().map(|x| add_to_insert(x)).collect::<Vec<String>>())),
        Err(e) => Err(e)
    }
}

pub fn parse_modifier(s: &str) -> IResult<&str,Vec<String>> {
    alt((
            parse_add_modifier,
            parse_delete_modifier
        ))(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_modifier() {
        assert_eq!(
            parse_add_modifier("add pierre ami jean").unwrap().1,
            vec!["INSERT INTO facts (subject,link,goal) VALUES (pierre,ami,jean)"]
                  );

        assert_eq!(
            parse_add_modifier("add pierre ami jean and julie ami susanne").unwrap().1,
            vec!["INSERT INTO facts (subject,link,goal) VALUES (pierre,ami,jean)",
                 "INSERT INTO facts (subject,link,goal) VALUES (julie,ami,susanne)"
            ]);
    } 

    #[test]
    fn test_triplet_to_insert() {
        assert_eq!(
            triplet_to_insert(&Twww("pierre","ami","jean")),
            "INSERT INTO facts (subject,link,goal) VALUES (pierre,ami,jean)".to_string());
    }

    #[test]
    fn test_triplet_and() {
        assert_eq!(
            parse_triplet_and(" B ami C AND A ami C").unwrap().1,
            Tri(Twww("B","ami","C"))
        );
    }

    #[test]
    fn test_delete_modifier() {
        assert_eq!(
            parse_delete_modifier("delete pierre ami jean").unwrap().1,
            vec!["DELETE FROM facts WHERE subject='pierre',link='ami',goal='jean'"]);

        assert_eq!(
            parse_delete_modifier("delete pierre ami jean and julie ami susanne").unwrap().1,
            vec!["DELETE FROM facts WHERE subject='pierre',link='ami',goal='jean'",
                 "DELETE FROM facts WHERE subject='julie',link='ami',goal='susanne'"]);
    }

}
