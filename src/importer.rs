//un importer peut être appellé depuis la ligne de commande par une option
//--importer [binary]

//L'utilisateur aura à préciser le chemin du binair à utiliser

#![allow(dead_code, unused_variables, unused_imports)]

use std::process::Command;

fn import(path: &str) {
    let _output = if cfg!(target_os = "windows") {
        Command::new("cmd")
                .args(["/C", path])
                .output()
                .expect("failed to execute process")
    } else {
        Command::new("sh")
                .arg("-c")
                .arg("./".to_owned()+path)
                .output()
                .expect("failed to execute process")
    };
}
