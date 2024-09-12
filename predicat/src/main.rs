#![allow(dead_code, unused_variables, unused_imports, unreachable_code, unused_assignments)]
use std::fs;
use rustyline::{Editor, Config, EditMode};
use rustyline::error::ReadlineError;
use rustyline::config::CompletionType;
use rustyline::history::DefaultHistory;
use rustyline::history::FileHistory;
use clap::{Command, Arg, ArgMatches};
use knowledge::SqliteKnowledge;
use knowledge::Knowledgeable;

mod interpreter;
use interpreter::Interpreter;
use parser::Cmd;


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

fn one_command(val: &String) -> () {
    let mut interpreter = Interpreter::new(SqliteKnowledge::new());
    interpreter.run(&val);
    interpreter.show();
}

fn process_string(input: &str) -> Vec<String> {
    let input_without_newlines = input.replace("\n", "");
    let mut vect = input_without_newlines
        .split(';')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
    vect.pop(); vect
}

fn read_file(val: &String) -> () {
    let val = open(val);
    let lines = process_string(&val);
    let mut interpreter = Interpreter::new(SqliteKnowledge::new());
    lines.iter().for_each(|cmd| {interpreter.run(cmd);});
    interpreter.show();
}

fn generate_shell() -> Editor<(), FileHistory> {
    let config = Config::builder()
        .edit_mode(EditMode::Emacs)
        .completion_type(CompletionType::List)
        .history_ignore_dups(true)
        .expect("Error with the shell method")
        .build();
    Editor::<(), DefaultHistory>::with_config(config)
        .expect("Erreur lors de l'initialisation de l'éditeur")
}

fn shell() {
    let mut rl = generate_shell();
    let mut interpreter = Interpreter::new(SqliteKnowledge::new());
    loop {
        let readline = rl.readline(&format!("{}> ", interpreter.get_mode()));
        match readline {
            Ok(x) if x == "exit" => break,
            Ok(x) if &x[0..5] == "parse" => break,
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                interpreter.run(&line);
                interpreter.show();
            },
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
        Some(("cmd", sub_matches)) => one_command(sub_matches.get_one::<String>("name")
                                                  .expect("No command where given as an argument")), 
        Some(("open", sub_matches)) => read_file(sub_matches.get_one::<String>("name")
                                                  .expect("No file name where given")), 
        Some(("shell", sub_matches)) => shell(),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}


