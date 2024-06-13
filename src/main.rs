#[allow(unused_imports)]
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn path_finder(command_name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH")?
        .to_str()?
        .split(":")
        .map(|elem| Path::new(elem).join(command_name))
        .find(|path| path.exists())
}

fn main() {
    let mut builtins: HashMap<&str, Box<dyn Fn(&str) -> i8>> = HashMap::new();
    builtins.insert(
        "exit",
        Box::new(move |input: &str| {
            let mut input = input.split_whitespace();
            let exit_code: i32 = input.next().unwrap_or(&"1").parse::<i32>().unwrap_or(1);
            std::process::exit(exit_code)
        }),
    );
    builtins.insert(
        "echo",
        Box::new(move |input: &str| {
            print!("{}\n", input);
            0
        }),
    );
    builtins.insert(
        "pwd",
        Box::new(move |_input: &str| {
            print!("{}\n", std::env::var("PWD").unwrap_or_default());
            0
        }),
    );
    builtins.insert(
        "cd",
        Box::new(move |input: &str| {
            if input == "~" || input == "" {
                #[allow(deprecated)]
                match std::env::home_dir() {
                    Some(path) => {
                        std::env::set_var("PWD", path.clone());
                        let _ = std::env::set_current_dir(path);
                    }
                    _ => {
                        print!("no home directory found\n");
                    }
                }
            } else {
                match std::path::Path::new(input).canonicalize() {
                    Ok(path) => {
                        if path.is_dir() {
                            std::env::set_var("PWD", path.clone());
                            let _ = std::env::set_current_dir(path);
                        }
                    }
                    Err(_) => {
                        print!("{}: No such file or directory\n", input);
                    }
                }
            }
            0
        }),
    );

    builtins.insert(
        "type",
        Box::new(move |input: &str| {
            let input = input.trim();
            if vec!["cd", "exit", "type", "echo", "pwd"]
                .into_iter()
                .find(|elem| *elem == input)
                .is_some()
            {
                print!("{} is a shell builtin\n", input);
                0
            } else {
                match path_finder(input) {
                    Some(path) => {
                        print!("{} is {}\n", input, path.display());
                        1
                    }
                    _ => {
                        print!("{} not found\n", input);
                        1
                    }
                }
            }
        }),
    );

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let (command, args) = input
            .trim()
            .split_once(char::is_whitespace)
            .unwrap_or((&input.trim(), ""));
        match builtins.get(command) {
            Some(function) => function(args),
            None => match path_finder(command) {
                Some(command) => {
                    if args != "" {
                        let code = std::process::Command::new(command)
                            .arg(args)
                            .status()
                            .unwrap_or_default();
                        code.code().unwrap() as i8
                    } else {
                        let code = std::process::Command::new(command)
                            .status()
                            .unwrap_or_default();
                        code.code().unwrap() as i8
                    }
                }
                _ => {
                    print!("{}: command not found\n", command);
                    1
                }
            },
        };
    }
}
