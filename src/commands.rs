use std::collections::HashMap;
use regex::Regex;


pub struct Param {
    value: String,
    validation: Regex
}

impl Param {
    fn new(value: &str, validation: &str) -> Param {
        Param {
            value: value.to_string(),
            validation: Regex::new(validation).unwrap()
        }
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: &str) -> bool {
        if self.validation.is_match(value) {
            self.value = value.to_string();
            true
        } else {
            false
        }
    }

    pub fn get_validation(&self) -> &Regex {
        &self.validation
    }

}


pub struct Commands {
    commands: HashMap<String, Param>
}

impl Commands {
    pub fn new() -> Commands {
        Commands {
            commands: HashMap::new()
        }
    }

    pub fn add(&mut self, name: &str, value: &str, validation: &str) {
        self.commands.insert(name.to_string(), Param::new(value, validation));

    }

}