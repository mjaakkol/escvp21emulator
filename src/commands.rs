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

    pub fn get_value(&self) -> Option<String> {
        if self.value.len() > 0 {
            Some(self.value.clone())
        } else {
            None
        }
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
    commands: HashMap<&'static str, Param>,
}

impl Commands {
    pub fn new() -> Commands {
        Commands {
            commands: HashMap::from([
                ("SNO",Param::new("1234567890","")),
                ("NAME",Param::new("EpsonArtome","")),
                ("FILT",Param::new("0000","")),
                ("LAMP",Param::new("0000","")),
                ("SOURCE",Param::new("0000","")),
                ("MODEL",Param::new("0000","")),
                ("VER",Param::new("0000","")),
                ("ERR",Param::new("0000","")),
                ("SOURCE",Param::new("0000","")),
                ("MUTE",Param::new("0000","")),
                ("VOL",Param::new("90","")),
                ("AVMT",Param::new("0000","")),
                ("AUTOHOME",Param::new("00","")),
                ("ZOOM",Param::new("0","\\d{1,3}")),
            ])
        }
    }
}