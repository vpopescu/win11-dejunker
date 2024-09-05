use log::{debug, warn};
use reqwest::blocking::get;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use yaml_rust2::YamlLoader;

use crate::types::onoff::OnOffType;
use crate::{registry, utils};

#[derive(Debug, Clone)]
pub struct Settings {
    pub file: String,
    pub settings: HashMap<String, String>,
}

pub const FILE_MARKER: &str = "redsigil.dfckr.settings.v1";

/// Read a settings file into memory
///
/// * path_or_url: the path of the settings file to read
///
pub fn read_settings_file(path_or_url: &str) -> Result<Settings, Box<dyn Error>> {
    // Read the content from either a local file or a URL
    let contents = if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        get(path_or_url)?.text()?
    } else {
        let mut file = File::open(path_or_url)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents
    };

    // Parse the YAML content
    let docs = YamlLoader::load_from_str(&contents)?;
    let doc = &docs[0];

    // Extract file and settings from the YAML
    let file = doc["file"]
        .as_str()
        .ok_or("Invalid input format, file is missing 'file' field")?
        .to_string();

    debug!("Settings file type is {}", file);
    if file != FILE_MARKER {
        panic!("Invalid database type when reading '{}'", file);
    }

    let settings_yaml = doc["settings"]
        .as_hash()
        .ok_or("Invalid input format, file is missing 'settings' field")?;
    let mut settings = HashMap::new();

    for (key, value) in settings_yaml {
        if let (Some(key), Some(value)) = (key.as_str(), value.as_str()) {
            settings.insert(key.to_string(), value.to_string());
        }
    }

    Ok(Settings { file, settings })
}

/// Execute a rule from the settings file
///
/// * rules: the list of known rules
/// * rule_name: the name of the rule to execute
/// * desired_value: the value to set the rule to
///
pub fn execute_rule(
    rules: &HashMap<String, crate::db::Rule>,
    rule_name: &str,
    desired_value: &str,
    skip_inaccessible: bool
) -> Result<(), Box<dyn std::error::Error>> {
    let rule = &rules[rule_name];

    if rule.admin_required && utils::is_elevated() == false {

        match skip_inaccessible  {
            true => {
                warn!("Rule {} was skipped, operation requires admin rights.", rule_name);
            },
            false => {
                return Err(format!("Rule {} requires admin rights.", rule_name).into());
            }
        }
       
        return Ok(());
    }

    for op in &rule.exec {
        match op.subsystem.as_str() {
            "registry" => {
                //let value = crate::registry::read_value(&op.path, &op.value, &op.value_type)?;

                match op.value_type.as_str() {
                    "i32" | "u32" => {
                        let desired_value = desired_value
                            .parse::<OnOffType>()
                            .expect(format!("Invalid value type for {}", rule_name).as_str());

                        let value = if op.reversed == Some(true) {
                            desired_value.flipped()
                        } else {
                            desired_value
                        }
                        .as_u32();

                        debug!("Setting {} -> {} to {}", op.path, op.value, value);
                        registry::set_u32_value(op.path.as_str(), op.value.as_str(), value)?;
                    }
                    _ => {
                        return Err(format!(
                            "Unsupported value type '{}' for '{}'",
                            op.value_type, op.path
                        )
                        .into());
                    }
                }
            }
            _ => {
                return Err(format!("Unsupported subsystem '{}'", op.subsystem).into());
            }
        }
    }

    Ok(())
}
