#![allow(dead_code, unused_variables, unused_imports, unreachable_code)]
use parser;
use std::env;
use knowledge;
use knowledge::Cache;
use parser::ContextCMD;
use serial_test::serial;
use parser::parse_command;
use knowledge::RuleManager;
use knowledge::Knowledgeable;
use knowledge::SqliteKnowledge;
use parser::base_parser::Triplet;
use parser::base_parser::PredicatAST;
use parser::base_parser::CommandType;
use base_context::context_traits::Context;
use metaprogramming::substitute_variables;
use base_context::simple_context::SimpleContext;

struct Interpreter {
    context: SimpleContext,
    knowledge: SqliteKnowledge
}

impl Interpreter {

    fn new(k: SqliteKnowledge) -> Interpreter {
        Interpreter {
            context: SimpleContext::new(),
            knowledge: k
        }
    }

    fn run(&mut self, cmd: &str) -> SimpleContext {
        
        let mut context = self.interpret(&vec![cmd.to_string()], &self.knowledge);

        while context.has_commands() && !context.has_error() {
            context = self.interpret(&context.get_aftercmds(), &self.knowledge); } self.context = context.clone();
        context
    }

    fn display(&self) -> () {
        self.context.display()
    }

    fn parse(command: &String) -> Vec<PredicatAST> {
        parse_command(command).iter()
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

    fn execute(&self, cmds: &[PredicatAST], knowledge: &impl Knowledgeable) -> Option<SimpleContext> {
        let context = knowledge
                .valid_commands(cmds.to_vec())?
                .iter()
                .map(|x| (x, knowledge.get_commands_from(x)))
                .map(|(cmd, aftcmd)| knowledge.execute_command(cmd).add_aftercmd(&aftcmd))
                .reduce(SimpleContext::join_contexts)?;

        Some(context.clone())
    }

    fn interpret(&self, cmds: &[String], knowledge: &impl Knowledgeable) -> SimpleContext {
        let cmds = cmds.iter()
                    .flat_map(Self::parse)
                    .collect::<Vec<PredicatAST>>();
        self.execute(&cmds, knowledge)
                    .expect("Something went wrong")
    }

    fn clear(&self) -> () {
        self.knowledge.clear_all();
    }
    
    fn get_rules(&self) -> Vec<String> {
        self.knowledge.get_rules()
    }

}

impl Default for Interpreter {
   fn default() -> Interpreter {
       Interpreter {
           context: SimpleContext::new(),
           knowledge: SqliteKnowledge::new()
       }
   } 
}

fn main() {
    //let knowledge = new_knowledge("sqlite").expect("Can't open the knowledge!");
    //let interpreter = Interpreter::default();
    //interpreter.run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use knowledge::base_knowledge::Command;

    #[test]
    #[serial]
    fn test_add(){
       let mut interpreter = Interpreter::default();
       interpreter.clear();
       interpreter.run("add julien ami julie");
       assert_eq!(
           SimpleContext::from(vec![["julien", "ami", "julie"]]),
           interpreter.run("get julien ami julie"));
    }

    #[test]
    #[serial]
    fn test_rule_1() {
       let mut interpreter = Interpreter::default();
       interpreter.clear();
       interpreter.run("infer add $A ami $B -> add $B ami $A");
       interpreter.run("add julien ami julie");
       interpreter.get_rules();
       assert_eq!(interpreter.get_rules(),
          vec!["add", "$A", "ami", "$B", "add $B ami $A", "add $B ami $A"]);
    }

    #[test]
    #[serial]
    fn test_rule_2() {
       let mut interpreter = Interpreter::default();
       interpreter.clear();
       interpreter.run("infer add $A ami $B -> add $B ami $A");
       interpreter.run("add julien ami julie");
        assert_eq!(
            SimpleContext::from(vec![["julien", "ami", "julie"],
                                    ["julie", "ami", "julien"]]),
            interpreter.run("get $subject $link $goal")
                  );
    }


    #[test]
    fn test_get_command_from() {
       let mut interpreter = Interpreter::default();
       interpreter.clear();
       interpreter.run("infer add $A ami $B -> add $B ami $A");
       let cmds = interpreter.knowledge
           .get_commands_from(&Interpreter::parse(&"add julien ami julie".to_string())[0]);
       assert_eq!(cmds,
                 ["add julie ami julien"]);
    }


}
