use std::env;
use std::path::Path;
use std::process::Command;

pub fn run(typ: &str, arg: &str) {
    match typ {
        "dots" => {
            if arg.is_empty() {
                eprintln!("{}", crate::colo::red("dots requires an argument"));
                std::process::exit(1);
            }
            let home = env::var("HOME").unwrap();
            let path = format!("{}/.owl/dotfiles/{}", home, arg);
            open_editor(&path);
        }
        "config" => {
            if arg.is_empty() {
                eprintln!("{}", crate::colo::red("config requires an argument"));
                std::process::exit(1);
            }
            let home = env::var("HOME").unwrap();
            let candidates = vec![
                format!("{}/.owl/{}", home, arg),
                format!("{}/.owl/{}.owl", home, arg),
                format!("{}/owl/hosts/{}", home, arg),
                format!("{}/owl/hosts/{}.owl", home, arg),
                format!("{}/owl/groups/{}", home, arg),
                format!("{}/owl/groups/{}.owl", home, arg),
            ];
            for candidate in candidates {
                if Path::new(&candidate).exists() {
                    open_editor(&candidate);
                    return;
                }
            }
            eprintln!("{}", crate::colo::red("config file not found"));
            std::process::exit(1);
        }
        _ => {
            eprintln!("{}", crate::colo::red("edit type must be dots or config"));
            std::process::exit(1);
        }
    }
}

fn open_editor(path: &str) {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    Command::new(editor)
        .arg(path)
        .status()
        .expect("failed to open editor");
}

