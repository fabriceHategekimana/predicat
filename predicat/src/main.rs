use std::fs;
use std::env;
use rustyline::{Editor, Config, EditMode};
use rustyline::error::ReadlineError;
use rustyline::config::CompletionType;
use rustyline::history::DefaultHistory;
use parser::ContextCMD;
use parser::parse_command;
use knowledge::Cache;
use knowledge::Knowledgeable;
use knowledge::SqliteKnowledge;
use clap::{Command, Arg, ArgMatches};
use parser::base_parser::PredicatAST;
use base_context::context_traits::Context;
use metaprogramming::substitute_variables;
use base_context::simple_context::SimpleContext;

struct Interpreter {
    context: SimpleContext,
    knowledge: SqliteKnowledge
}

impl Interpreter {

    fn run(&mut self, cmd: &str) -> SimpleContext {
        
        let mut context = self.interpret(&vec![cmd.to_string()], &self.knowledge);
        while context.has_commands() && !context.has_error() {
            context = self.interpret(&context.get_aftercmds(), &self.knowledge); } self.context = context.clone();
        self.clear_cache();
        self.context = context.clone();
        context
    }

    fn clear_cache(&self) -> () {
        self.knowledge.clear_cache();
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
                .filter(|x| !knowledge.in_cache(x))
                .map(|x| (x, knowledge.get_commands_from(x)))
                .map(|(cmd, aftcmd)| knowledge.execute_command(cmd).add_aftercmd(&aftcmd))
                .reduce(SimpleContext::join_contexts)?;
        Some(context.clone())
    }

    fn interpret(&self, cmds: &[String], knowledge: &impl Knowledgeable) -> SimpleContext {
        let cmds = cmds.iter()
                    .flat_map(Self::parse)
                    .collect::<Vec<PredicatAST>>();
        self.execute(&cmds, knowledge).unwrap_or_default()
    }

    fn clear(&self) -> () {
        self.knowledge.clear_all();
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


fn open(file_name: &str) -> String {
    fs::read_to_string(file_name)
                .expect(&format!("le fichier '{}' est illisible", file_name))
}

fn get_user_input() -> ArgMatches {
    Command::new("MyApp")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("cmd")
                .about("Run a command passed as a parameter")
                .arg(Arg::new("name"))
        )
        .subcommand(
            Command::new("open")
                .about("Open a file and execute its predicat's comment")
                .arg(Arg::new("name"))
                   )
        .subcommand(
            Command::new("shell")
                .about("Execute an interactive shell for predicat")
                   )
        .get_matches()
}

fn one_command(val: Option<&String>) -> () {
    let mut interpreter = Interpreter::default();
    //let args = interpreter.get_args_or("add socrate est mortel");
    interpreter.run(&val.expect("No command where given as an argument"));
    interpreter.display();
}


fn process_string(input: &str) -> Vec<String> {
    let input_without_newlines = input.replace("\n", "");
    input_without_newlines
        .split(';')
        .map(|s| s.trim().to_string())
        .collect()
}

fn read_file(val: Option<&String>) -> () {
    let val = open(val.expect("No file name was given"));
    let lines = process_string(&val);
    let mut interpreter = Interpreter::default();
    lines.iter().for_each(|cmd| {interpreter.run(cmd);});
    interpreter.display();
}

fn shell() {
    let config = Config::builder()
        .edit_mode(EditMode::Emacs)
        .completion_type(CompletionType::List)
        .history_ignore_dups(true)
        .expect("Error with the shell method")
        .build();
    let mut rl = Editor::<(), DefaultHistory>::with_config(config)
        .expect("Erreur lors de l'initialisation de l'éditeur");

    loop {
        //let mut rl = rustyline::DefaultEditor::new().unwrap();
        let readline = rl.readline(">> ");
        match readline {
            Ok(exit) if exit == "exit" => break,
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                one_command(Some(&line))},
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(_) => println!("No input"),
        }
    }
}

fn main() {
    match get_user_input().subcommand() {
        Some(("cmd", sub_matches)) => one_command(sub_matches.get_one::<String>("name")), 
        Some(("open", sub_matches)) => read_file(sub_matches.get_one::<String>("name")),
        Some(("shell", sub_matches)) => shell(),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knowledge::base_knowledge::Command;
    use serial_test::serial;
    use knowledge::RuleManager;

    impl Interpreter {
        fn get_rules(&self) -> Vec<String> {
            self.knowledge.get_rules()
        }
    }

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
            interpreter.run("get $subject $link $goal where $subject $link $goal")
                  );
    }

    #[test]
    #[serial]
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
