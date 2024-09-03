use log::{debug, error};
use reqwest::blocking::get;
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, fs::read_to_string, process};
use yaml_rust2::YamlLoader;

const DATABASE_ID: &str = "redsigil.dfckr.db.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesDatabase {
    pub file: String,
    pub rules: HashMap<String, Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub admin_required: bool,
    pub value: Value,
    pub exec: Vec<Exec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    #[serde(rename = "type")]
    pub value_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exec {
    pub subsystem: String,
    pub path: String,
    pub value: String,
    pub value_type: String,
    pub reversed: Option<bool>,
}

/// Read a rules database into memory
/// 
/// * path_or_url: the path of the rule file to read
/// 
pub fn read_database(path_or_url: &str) -> Result<RulesDatabase, Box<dyn std::error::Error>> {
    let contents = if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        match get(path_or_url) {
            Ok(response) => response.text()?,
            Err(e) => {
                error!(
                    "Could not fetch database from URL '{}': {}",
                    path_or_url,
                    e.to_string()
                );
                process::exit(1);
            }
        }
    } else {
        match read_to_string(path_or_url) {
            Ok(contents) => contents,
            Err(e) => {
                error!(
                    "Could not read database file '{}': {}",
                    path_or_url,
                    e.to_string()
                );
                process::exit(e.raw_os_error().unwrap_or(1));
            }
        }
    };

    let docs = YamlLoader::load_from_str(&contents)?;
    let doc = &docs[0];

    let file = doc["file"].as_str().unwrap_or("").to_string();

    debug!("Database type is {}", file);
    if file != DATABASE_ID {
        panic!("Invalid database type when reading '{}'", file);
    }

    let rules_yaml: &Vec<yaml_rust2::Yaml> = doc["rules"].as_vec().ok_or("Invalid rules format")?;

    let mut rules = HashMap::<String, Rule>::new();
    for rule_yaml in rules_yaml {
        let rule = Rule {
            id: rule_yaml["rule"].as_str().unwrap_or("").to_string(),
            name: rule_yaml["arg"].as_str().unwrap_or("").to_string(),
            description: rule_yaml["description"].as_str().unwrap_or("").to_string(),
            admin_required: rule_yaml["admin_required"].as_bool().unwrap_or(false),
            value: Value {
                value_type: rule_yaml["value"]["type"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            },
            exec: rule_yaml["exec"]
                .as_vec()
                .unwrap_or(&vec![])
                .iter()
                .map(|exec_yaml| Exec {
                    subsystem: exec_yaml["subsystem"].as_str().unwrap_or("").to_string(),
                    path: exec_yaml["path"].as_str().unwrap_or("").to_string(),
                    value: exec_yaml["value"].as_str().unwrap_or("").to_string(),
                    value_type: exec_yaml["type"].as_str().unwrap_or("").to_string(),
                    reversed: exec_yaml["reversed"].as_bool(),
                })
                .collect(),
        };
        debug!("Read rule {}", rule.name);
        rules.insert(rule.name.clone(), rule);
    }

    debug!("Read {} rules", rules.len());
    let result = RulesDatabase { file, rules };

    Ok(result)
}
