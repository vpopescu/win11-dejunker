use clap::{Arg, ArgGroup, Command};
use files::db::{self, Exec};
use log::{debug, error};
use std::{collections::HashMap, env, fs::File, io::Write, process};

mod files;
mod registry;
mod types;
mod utils;

use types::onoff::OnOffType;

const DEFAULT_DB: &str = "db.yaml";

#[cfg(windows)]
const DELIM: &str = "\r\n";

#[cfg(not(windows))]
const DELIM: &str = "\n";

/// Main application function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // add some standard options
    let mut cmd = Command::new("Windows dejunker")
        .version("1.0.0")
        .about("Fix windows 11")
        .arg(
            Arg::new("input")
                .conflicts_with("output")
                .long("input-file")
                .value_name("input rules file")
                .short('i')
                .required(false)
                .require_equals(false)
                .display_order(0)
                .help("Settings file to be applied"),
        )
        .arg(
            Arg::new("output")
                .conflicts_with("input")
                .long("output-file")
                .short('o')
                .value_name("output rules file")
                .required(false)
                .require_equals(false)
                .display_order(1)
                .help("Write settings to this file"),
        )
        .arg(
            Arg::new("db")
                .long("database-file")
                .required(false)
                .short('s')
                .default_value(DEFAULT_DB)
                .value_name("rules database")
                .require_equals(false)
                .display_order(2)
                .help("Database file (definitions of known settings)"),
        )
        .group(ArgGroup::new("opts").required(false).multiple(true));

    let pre_matches = cmd.clone().try_get_matches();

    let db = if pre_matches.is_ok() {
        pre_matches
            .unwrap()
            .get_one::<String>("db")
            .unwrap()
            .clone()
    } else {
        DEFAULT_DB.to_owned()
    };

    let rules = db::read_database(&db).expect("Unable to parse YAML").rules;
    let mut commands: Vec<&str> = Vec::new();
    commands.push("input");
    commands.push("output");

    // dynamically add all rules as arguments

    let mut display_order: usize = 4;

    for rule in rules.values() {
        let arg_name: &'static str = Box::leak(rule.name.clone().into_boxed_str());
        let description: &'static str = Box::leak(rule.description.clone().into_boxed_str());

        let (val, values) = if rule.value.value_type.to_lowercase() == "onoff" {
            (clap::value_parser!(OnOffType), "on|off")
        } else {
            error!("Value type {} is not supported", &rule.value.value_type);
            process::exit(1);
        };

        // add to opts
        cmd = cmd.arg(
            Arg::new(arg_name)
                .conflicts_with_all(&["input", "output"])
                .display_order(display_order)
                .require_equals(true)
                .long(arg_name)
                .value_parser(val)
                .value_name(values)
                .group("opts")
                .help(description),
        );

        commands.push(arg_name);
        display_order += 1;
    }

    let matches = cmd.get_matches();
    let args_supplied = env::args().len() > 1;

    // if output_file is specified or no parameters are specified, this is view mode
    let output_file = matches.get_one::<String>("output");
    let input_file = matches.get_one::<String>("input");

    let mut accumulator: String = format!(
        "file: {}{}settings: {}",
        files::settings::FILE_MARKER.to_owned(),
        DELIM.to_owned(),
        DELIM.to_owned()
    );

    // read  mode
    if !args_supplied || output_file.is_some() {
        accumulator += print_values(&rules, output_file)?.as_str();
        if !accumulator.is_empty() {
            match output_file {
                Some(_) => {
                    write_string_to_file(accumulator.as_str(), output_file.unwrap().as_str())?;
                }
                None => {
                    println!("{}", accumulator);
                }
            }
        }
    }

    // write mode (file)
    if input_file.is_some() {
        apply_settings_file(&rules, input_file.unwrap().as_str())?;
    }

    // get all supplied args
    for arg in commands.iter() {
        if matches.contains_id(arg) {
            let value = matches.get_one::<OnOffType>(arg).unwrap();
            files::settings::execute_rule(&rules, arg, &value.to_string())?;
        }
    }
    Ok(())
}

// print accumulated string to output file, or stdout
///
/// * rules: the list of rules to be printed
/// * output_file: the file to write to (stdout if None)
///
fn print_values(
    rules: &HashMap<String, db::Rule>,
    output_file: Option<&String>,
) -> Result<String, Box<dyn std::error::Error>> {
    debug!(
        "Output file is {}",
        output_file.unwrap_or(&String::from("stdout"))
    );

    let mut output = String::new();
    for rule in rules.values() {
        let arg_name = rule.name.clone();
        output.push_str(&format!(
            "    {}: {}\n",
            arg_name,
            evaluate_rule(&rule.exec)?
        ));
    }
    Ok(output)
}

/// Check the value of a rule
///
/// * exec: The native part of the rule (usually registry access information)
///
fn evaluate_rule(exec: &Vec<Exec>) -> Result<String, Box<dyn std::error::Error>> {
    let mut results: HashMap<OnOffType, i32> = HashMap::new();

    for op in exec.iter() {
        match op.subsystem.as_str() {
            "registry" => {
                let value = registry::read_value(&op.path, &op.value, &op.value_type)?;

                match op.value_type.as_str() {
                    "i32" => {
                        let mut value = OnOffType::from_string(value.as_str());
                        if op.reversed.unwrap_or(false) {
                            value = value.flipped();
                        }

                        // all values must evaluate to On or Off. If some evaluate to on and some to off, we assume off.
                        // We use a hashmap as a lazy way of determining this
                        results.insert(value, 0);
                    }
                    _ => {
                        error!("Value type {} is not supported", op.value_type);
                        process::exit(1);
                    }
                };
            }
            _ => {
                error!("Subsystem '{}' is not supported", op.subsystem);
                process::exit(1);
            }
        }
    }

    if results.keys().len() == 1 {
        return Ok(results.keys().next().unwrap().to_string());
    } else {
        return Ok(OnOffType::On.to_string());
    }
}

// write accumulated string to file
fn write_string_to_file(content: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Read settings from a settings file, and apply the directives in that file
///
/// * rules: the list of known rules
/// * path_or_url: the settings files to apply
///
fn apply_settings_file(
    rules: &HashMap<String, db::Rule>,
    path_or_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = files::settings::read_settings_file(path_or_url)?;

    // for each setting
    for (key, value) in file.settings.iter() {
        files::settings::execute_rule(rules, &key, &value)?;
    }

    Ok(())
}
