#![allow(dead_code, unused_variables, unused_imports)]

mod parse_query;

use parse_query::*;

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

fn parse_modifier(s: &str) -> IResult<&str,Vec<String>> {
    alt((
            parse_add_modifier,
            parse_delete_modifier
        ))(s)
}

//main
pub fn parse_command(s: &str) -> Vec<String> {
    let res = alt((
            parse_query,
            parse_modifier))(s);
    match res {
        Ok((s, t)) => t,
        Err(e) => vec![String::from("Error")] //;format!("{}", Err(e))
    }
}

fn parse_add(s: &str) -> String {
    let res = many1(parse_triplet_and)(s);
    match res {
        Ok((t, v)) => "a".to_string(),
        _ => "b".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query() {
        assert_eq!(parse_query("get $A such_as $A ami Bob $A == 7").unwrap().1,
                   //(vec![Var("A")], vec![Tri(Tvww("A","ami","Bob"))], vec![Comp(" $A == 7")])
                   vec!["SELECT A FROM (SELECT subject AS A FROM facts WHERE link='ami' AND goal='Bob') WHERE A = 7;"]
                   );
        assert_eq!(parse_query("get $A $B such_as $A ami $B and $A == 7").unwrap().1,
                   //(vec![Var("A"), Var("B")], vec![Tri(Tvwv("A","ami","B"))], vec![Comp(" $A == 7")])
                   vec!["SELECT A,B FROM (SELECT subject AS A,goal AS B FROM facts WHERE link='ami') WHERE A = 7;"]
                   );
        assert_eq!(parse_query("get $A $B such_as $A ami $B and $A == 7 and $B < 9").unwrap().1,
                   //(vec![Var("A"), Var("B")], vec![Tri(Tvwv("A","ami","B"))], vec![Comp(" $A == 7"), Comp(" $B < 9")])
                   vec!["SELECT A,B FROM (SELECT subject AS A,goal AS B FROM facts WHERE link='ami') WHERE A = 7 AND B < 9;"]
                   );
    }

    #[test]
    fn test_triplet_and() {
        assert_eq!(
            parse_triplet_and(" B ami C AND A ami C").unwrap().1,
            Tri(Twww("B","ami","C"))
        );
    }


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
    fn test_delete_modifier() {
        assert_eq!(
            parse_delete_modifier("delete pierre ami jean").unwrap().1,
            vec!["DELETE FROM facts WHERE subject='pierre',link='ami',goal='jean'"]);

        assert_eq!(
            parse_delete_modifier("delete pierre ami jean and julie ami susanne").unwrap().1,
            vec!["DELETE FROM facts WHERE subject='pierre',link='ami',goal='jean'",
                 "DELETE FROM facts WHERE subject='julie',link='ami',goal='susanne'"]);
    }

    #[test]
    fn test_triplet_to_insert() {
        assert_eq!(
            triplet_to_insert(&Twww("pierre","ami","jean")),
            "INSERT INTO facts (subject,link,goal) VALUES (pierre,ami,jean)".to_string());
    }
}
