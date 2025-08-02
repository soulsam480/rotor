use regex::{Regex, RegexBuilder};
use std::{fs, io};

struct SecretValue {
    value: String,
    name: String,
    secret: String,
}

/*
* TODO:
* 1. check if the config is there and ask them to init if it's not there
* 2. figure out from the file, what are all the secret names to be set // done
* 3. for each secret, what are all values that can be set and their names // done
* 4. - when it's called, we need to list all secrets
*    - we need to ask which secret to look at
*    - when a secret is selected, we need to ask what value to set
*    - set the value for the secret and make it available in the terminal (echo $OPENAI_KEY)
* */
fn main() {
    let home_dir = env!("HOME");

    let config_path = format!("{}/.secretsrc", home_dir);

    let has_file = fs::exists(&config_path);

    if has_file.is_err() || !has_file.unwrap() {
        println!("Config doesn't exist bitch!!");
        return;
    }

    let file = fs::read_to_string(&config_path);

    match file {
        Ok(content) => {
            let mut secrets: Vec<String> = parse_secret_names(&content);

            let secret_values: Vec<SecretValue> = parse_secret_options(&content);

            let sec_index: usize = greet(&secrets).try_into().unwrap();

            let secret_name = secrets.get(sec_index).unwrap();

            let possible_values: Vec<&SecretValue> = secret_values
                .iter()
                .filter(|it| it.secret == *secret_name)
                .collect();

            let value_names: String = possible_values
                .iter()
                .map(|it| it.name.clone())
                .collect::<Vec<String>>()
                .join("\n");

            println!(
                "Choose a secret\n{}\nEnter a number from 0 - {}",
                value_names,
                possible_values.len() - 1
            )
        }

        _ => {
            println!("Error reading file");
        }
    }
}

fn greet(secrets: &Vec<String>) -> u32 {
    println!(
        "Welcome to Rotor!.\nPlease choose a secret from below to proceed.\n{}\nenter number 0 - {}",
        secrets.join("\n"),
        secrets.len() - 1
    );

    let mut secret_index = String::new();

    io::stdin().read_line(&mut secret_index).unwrap();

    let index: u32 = secret_index.trim().parse().unwrap();

    index
}

fn parse_secret_options(file: &str) -> Vec<SecretValue> {
    let values_re = RegexBuilder::new(r"--values\n(.*)\n--values")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    let secret_value_re = Regex::new(r"^(?<label>\w+):(?<secret_name>\w+)=([\w]+)").unwrap();

    values_re
        .captures(&file)
        .map(|it| it.get(1))
        .flatten()
        .map(|it| {
            it.as_str()
                .split("\n")
                .filter(|it| it.len() > 0)
                .map(|row| {
                    // NOTE: label:name=value <- row should be like this here
                    let (label, name, value): (&str, &str, &str) = secret_value_re
                        .captures(row)
                        .map(|captures| {
                            let (_, [label, name, value]) = captures.extract();

                            (label, name, value)
                        })
                        .unwrap();

                    let sec_val = SecretValue {
                        secret: name.to_string(),
                        value: value.to_string(),
                        name: label.to_string(),
                    };

                    sec_val
                })
        })
        .unwrap()
        .collect()
}

fn parse_secret_names(file: &str) -> Vec<String> {
    let schema_re = RegexBuilder::new(r"--secrets\n(.*)\n--secrets")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    schema_re
        .captures(&file)
        .map(|it| it.get(1))
        .flatten()
        .map(|it| {
            it.as_str()
                .split("\n")
                .filter(|it| it.len() > 0)
                .map(|name| name.to_string())
        })
        .unwrap()
        .collect()
}
