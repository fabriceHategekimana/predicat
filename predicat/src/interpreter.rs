use base_context::simple_context::SimpleContext;
use base_context::dataframe::DataFrame;
use parser::base_parser::PredicatAST;
use parser::cmd::Cmd;
use std::env;
use std::fmt;
use std::fmt::Debug;

use parser::parse_command;
use knowledge::Cache;
use knowledge::Knowledgeable;
use knowledge::SqliteKnowledge;
use base_context::context_traits::Context;
use metaprogramming::substitute_variables;
use base_context::context_cmd::ContextCMD;

#[derive(Debug, Clone)]
pub enum Mode {
    Normal,
    Inference,
    Parse,
    Sql
}

impl From<&str> for Mode {
   fn from(val: &str) -> Self {
        match val {
            "inference" => Mode::Inference,
            "parse" => Mode::Parse,
            "sql" => Mode::Sql,
            _ => Mode::Normal
        }
   } 
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            Mode::Normal => "normal",
            Mode::Inference => "inference",
            Mode::Parse => "parse",
            Mode::Sql => "sql",
        };
        write!(f, "{}", res)
    }
}

#[derive(Debug)]
pub struct Interpreter<K: Knowledgeable<DataFrame>> {
    context: SimpleContext,
    knowledge: K,
    mode: Mode
}

impl<K: Knowledgeable<DataFrame>> Interpreter<K> {

    pub fn new(k: K) -> Self {
        Interpreter { 
            context: SimpleContext::default(),
            knowledge: k,
            mode: Mode::Normal
            }
    }

    fn propagate(&mut self, ctx: SimpleContext) -> SimpleContext {
        let mut context = ctx;
        while context.has_commands() && !context.has_error() {
            context = Some(&context.get_aftercmds())
                    .map(|x| self.parse(x))
                    .map(|x| self.execute(&x))
                    .unwrap().unwrap_or_default()
        } self.context = context.clone(); self.clear_cache();
        context.clone()
    }

    pub fn run(&mut self, cmd: &str) -> SimpleContext {
        if Self::start_with(cmd, "mode") {
            self.mode = Mode::from(&cmd[5..]);
            SimpleContext::new()
        } else {
            match self.mode {
                Mode::Normal => self.full_run(cmd),
                Mode::Parse => self.parse_only(cmd),
                Mode::Sql => self.sql_only(cmd),
                Mode::Inference => self.infer_only(cmd),
            }
        }
    }

    fn infer_only(&self, cmd: &str) -> SimpleContext {
        let infered: Vec<_> = Some(&vec![Cmd::new(cmd)])
            .map(|x| self.parse(x))
            .unwrap().iter()
            .map(|cmd| self.knowledge.infer_commands_from(cmd))
            .collect();
        dbg!(infered);
        SimpleContext::new()
    }

    fn sql_only(&self, cmd: &str) -> SimpleContext {
        let sql: Vec<_> = Some(&vec![Cmd::new(cmd)])
            .map(|x| self.parse(x))
            .map(|adts| adts.iter().map(|x| self.knowledge.translate(x)).collect())
            .unwrap();
        dbg!(sql);
        SimpleContext::new()
    }

    fn parse_only(&self, cmd: &str) -> SimpleContext {
        let adt = Some(&vec![Cmd::new(cmd)])
            .map(|x| self.parse(x))
            .unwrap();
        dbg!(adt);
        SimpleContext::new()
    }

    fn full_run(&mut self, cmd: &str) -> SimpleContext {
        Some(&vec![Cmd::new(cmd)])
            .map(|x| self.parse(x))
            .map(|x| self.execute(&x).unwrap_or_default())
            //.map(|x| self.propagate(x))
            .unwrap()
    }

    fn inference(&mut self, cmd: &str) -> SimpleContext {
        let res = Some(&vec![Cmd::new(cmd)])
            .map(|x| self.parse(x))
            .map(|cmds| {
                let context = self.knowledge
                    .valid_commands(cmds.to_vec()).unwrap().iter()
                    .filter(|cmd| !self.knowledge.in_cache(cmd))
                    .map(|cmd| self.knowledge.infer_commands_from(cmd))
                    .collect::<Vec<_>>();
                })
            .unwrap();
        println!("res: {:?}", res);
        SimpleContext::new()
    }

    fn start_with(cmd: &str, term: &str) -> bool {
        if cmd.len() <= term.len() {
            false
        } else {
            &cmd[..(term.len())] == term
        }
    }

    fn clear_cache(&self) -> () {
        self.knowledge.clear_cache();
    }

    pub fn show(&self) -> () {
        self.context.show()
    }

    fn single_parse(command: &str) -> Vec<PredicatAST> {
        parse_command(&Cmd::new(&command)).iter()
                    .map(PredicatAST::clone)
                    .flat_map(substitute_variables(SimpleContext::new()))
                    .flatten().collect()
    }

    fn get_user_passed_arguments(&self) -> String {
        env::args().skip(1)
            .fold(String::new(), |acc, arg| format!("{}{} ", acc, &arg))
    }

    fn get_args_or(&self, query: &str) -> String {
        let args = self.get_user_passed_arguments();
        if args == "".to_string() {
            String::from(query)
        }
        else{
            args
        }
    }

    fn execute(&self, cmds: &[PredicatAST]) -> Option<SimpleContext> {
        let context = self.knowledge
                .valid_commands(cmds.to_vec())?.iter()
                .filter(|cmd| !self.knowledge.in_cache(cmd))
                .map(|cmd| (cmd, self.knowledge.infer_commands_from(cmd)))
                .map(|(cmd, aftcmd)| {
                        let context: SimpleContext = self.knowledge.execute_command(cmd).into();
                        let vals = aftcmd.iter().map(|x| Cmd::new(x)).collect::<Vec<_>>();
                            context.add_aftercmd(&vals)
                })
                .reduce(SimpleContext::join_contexts)?;
        Some(context.clone())
    }

    fn parse(&self, cmds: &[Cmd]) -> Vec<PredicatAST> {
        cmds.iter().flat_map(|c| Self::single_parse(&**c)).collect()
    }

    pub fn clear(&self) -> () {
        self.knowledge.clear_all();
    }

    pub fn get_mode(&self) -> Mode {
        self.mode.clone()
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use knowledge::base_knowledge::Command;
    use serial_test::serial;
    use knowledge::RuleManager;

    #[test]
    #[serial]
    fn test_add(){
       let mut interpreter = Interpreter::new(SqliteKnowledge::new());
       interpreter.clear();
       interpreter.run("add julien ami julie");
       assert_eq!(
           SimpleContext::from(vec![["julien", "ami", "julie"]]),
           interpreter.run("get julien ami julie"));
    }

    //#[test]
    //#[serial]
    //fn test_rule_2() {
       //let mut interpreter = Interpreter::new(SqliteKnowledge::new());
       //interpreter.clear();
       //interpreter.run("infer add $A ami $B -> add $B ami $A");
       //interpreter.run("add julien ami julie");
        //assert_eq!(
            //SimpleContext::from(vec![["julien", "ami", "julie"],
                                    //["julie", "ami", "julien"]]),
            //interpreter.run("get $subject $link $goal where $subject $link $goal")
                  //);
    //}
}

