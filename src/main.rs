use core::fmt;
use home;
use regex::{Regex, RegexBuilder};
use std::io::Write;
use std::path::PathBuf;
use std::{
    env,
    fs::{self, OpenOptions},
    io::{self},
    process,
};

struct SecretValue {
    value: String,
    name: String,
    secret: String,
}

#[derive(Debug)]
enum AppError {
    HomeDirNotFound,
    Io(io::Error),
    System(String),
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::HomeDirNotFound => write!(f, "Home Dir not found"),
            Self::Io(msg) => write!(f, "IO Error: {}", msg),
            Self::System(msg) => write!(f, "System Error: {}", msg),
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:?}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    let command = env::args().nth(1);

    match command {
        Some(comm) => {
            if comm.eq("init") {
                return run_init();
            }

            if comm.eq("help") {
                run_help();
                return Ok(());
            }

            Ok(())
        }

        _ => run_main(),
    }
}

fn run_help() {
    print!(
        "Welcome to rotor.
A shell secrets rotator. Quickly set a value for a secret based on predefined values.

Commands:
 init: setup rotor shell function to use the rotator.
 help: print this message.
"
    )
}

fn run_init() -> Result<(), AppError> {
    let home_dir = home::home_dir().ok_or(AppError::HomeDirNotFound)?;

    let config_path = home_dir.join(".zshrc");

    println!("Writing rotor shell function to: {}", config_path.display());

    let mut file = OpenOptions::new()
        .append(true)
        .create(false)
        .open(config_path)?;

    writeln!(
        file,
        "\nrotor() {{
  eval \"$(rotor_bin \"$@\")\"
}}"
    )?;

    let secrets_path = get_secrets_config()?;

    let secrets_rc_exists = fs::exists(&secrets_path)?;

    if !secrets_rc_exists {
        println!(
            "Secrets config doesn't exist. writing to {}",
            secrets_path.display()
        );

        fs::write(
            secrets_path,
            "--secrets
DEMO_KEY
--secrets

--values
label_1:DEMO_KEY=\"value_1\" 
label_2:DEMO_KEY=\"value_2\" 
label_3:DEMO_KEY=\"value_3\" 
--values",
        )?;
    }

    Ok(())
}

fn get_secrets_config() -> Result<PathBuf, AppError> {
    let home_dir = home::home_dir().ok_or(AppError::HomeDirNotFound)?;

    let config_path = home_dir.join(".secretsrc");

    Ok(config_path)
}

fn run_main() -> Result<(), AppError> {
    let file = fs::read_to_string(&get_secrets_config()?)?;

    let secrets: Vec<String> = parse_secret_names(&file)?;

    let secret_values: Vec<SecretValue> = parse_secret_options(&file)?;

    let secret_index = greet_and_ask_secret(&secrets)?;

    let chosen_secret = secrets.get(secret_index).ok_or_else(|| {
        AppError::System(String::from("Unable to find secret for provided index!"))
    })?;

    let possible_values: Vec<&SecretValue> = secret_values
        .iter()
        .filter(|it| it.secret.eq(chosen_secret))
        .collect();

    let value_index = print_secret_values(chosen_secret, &possible_values)?;

    let value_to_set = possible_values.get(value_index).ok_or_else(|| {
        AppError::System(String::from("Unable to find a value for provided index"))
    })?;

    eprintln!("Setting {} value for {}", value_to_set.name, chosen_secret);

    println!("export {}={}", chosen_secret, value_to_set.value);

    Ok(())
}

fn print_secret_values(secret: &str, secret_values: &[&SecretValue]) -> Result<usize, AppError> {
    eprintln!(
        "Choose a value to set for {}.\n{}:",
        secret,
        secret_values
            .iter()
            .map(|it| it.name.as_str())
            .collect::<Vec<&str>>()
            .join("\n")
    );

    loop {
        eprintln!("Enter a number from 0 to {}:", secret_values.len() - 1);

        let mut value_index = String::new();

        io::stdin().read_line(&mut value_index)?;

        match value_index.trim().parse::<usize>() {
            Ok(index) if index < secret_values.len() => return Ok(index),

            _ => {
                eprintln!("Invalid choice, Please try again");
            }
        }
    }
}

fn greet_and_ask_secret(secrets: &[String]) -> Result<usize, AppError> {
    eprintln!(
        "Welcome to Rotor!.\nPlease choose a secret from below to proceed.\n{}",
        secrets.join("\n"),
    );

    loop {
        eprintln!("Please enter a number from 0 to {}:", secrets.len() - 1);

        let mut secret_index = String::new();

        io::stdin().read_line(&mut secret_index)?;

        match secret_index.trim().parse::<usize>() {
            Ok(index) if index < secrets.len() => return Ok(index),

            _ => {
                eprintln!("Invalid choice, please try again!")
            }
        }
    }
}

fn parse_secret_options(file: &str) -> Result<Vec<SecretValue>, AppError> {
    let values_re = RegexBuilder::new(r"--values\n(.*)\n--values")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .map_err(|_| AppError::System(String::from("Unable to build secret value regex")))?;

    let secret_value_re = Regex::new(r#"^(?<label>\w+):(?<secret_name>\w+)=([\w"]+)"#)
        .map_err(|_| AppError::System(String::from("Unable to build secret value regex")))?;

    let value_captures = values_re
        .captures(file)
        .ok_or_else(|| AppError::System(String::from("Unable to find any secret")))?;

    let content = value_captures
        .get(1)
        .ok_or_else(|| AppError::System(String::from("Unable to find any secret values")))
        .map(|it| it.as_str().lines())?;

    let mut values: Vec<SecretValue> = vec![];

    for row in content {
        if row.is_empty() {
            continue;
        }

        let value = secret_value_re
            .captures(row)
            .ok_or_else(|| AppError::System(format!("Unable to parse row {}", row)))
            .map(|it| {
                let (_, [label, name, value]) = it.extract();

                SecretValue {
                    secret: name.to_string(),
                    value: value.to_string(),
                    name: label.to_string(),
                }
            })?;

        values.push(value);
    }

    Ok(values)
}

fn parse_secret_names(file: &str) -> Result<Vec<String>, AppError> {
    let schema_re = RegexBuilder::new(r"--secrets\n(.*)\n--secrets")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .map_err(|_| AppError::System(String::from("Unable to build secrets base regex")))?;

    let captures = schema_re
        .captures(file)
        .ok_or_else(|| AppError::System(String::from("Unable to find any secrets")))?;

    let content = captures
        .get(1)
        .ok_or_else(|| AppError::System(String::from("Unable to find any secrets")))?;

    let names = content
        .as_str()
        .lines()
        .filter(|it| !it.is_empty())
        .map(|it| it.to_string())
        .collect();

    Ok(names)
}
