use std::{
    io::{Read, Write},
    time
};

use crate::codec;

// Timeouts
const WARMING_TIME: time::Duration = time::Duration::from_secs(5);
const COOLDOWN_TIME: time::Duration = time::Duration::from_secs(3);

#[derive(Debug)]
pub enum PowerState {
    PowerOff,
    Warming(time::SystemTime),
    Cooling(time::SystemTime),
    LampOn,
    Terminated,
}


impl PowerState {
    pub fn new() -> PowerState {
        PowerState::PowerOff
    }

    pub fn process_message(&mut self, message: &str) -> Result<Option<String>, std::io::Error> {
        if message == "PWR?\r" {
            match self {
                // This ensures that timer effect becomes visible if expired
                PowerState::Warming(timer) => {
                    if timer.elapsed().unwrap() > WARMING_TIME {
                        *self = PowerState::LampOn;
                    }
                },
                PowerState::Cooling(timer) => {
                    if timer.elapsed().unwrap() > COOLDOWN_TIME {
                        *self = PowerState::PowerOff;
                    }
                },
                _ => (),
            }

            return Ok(Some(self.get_state_string(message)))
        } else {
            match self {
                PowerState::PowerOff => self.process_poweroff(message),
                PowerState::Warming(time_stamp) => self.process_warming(message),
                PowerState::Cooling(time_stamp) => self.process_cooling(message),
                PowerState::LampOn => self.process_lamp_on(message),
                _ => {
                    println!("Process message: invalid state: {:?}", self);
                    self.process_poweroff(message)
                }
            }
        }
    }

    pub fn get_state_string(&self, message: &str) -> String {
        format!("PWR={}\r",
            match self {
                PowerState::PowerOff => "00",
                PowerState::Warming(_) => "01",
                PowerState::Cooling(_) => "03",
                PowerState::LampOn => "02",
                PowerState::Terminated => "99",
            }
        )
    }

    pub fn process_cooling(&mut self, message: &str) -> Result<Option<String>, std::io::Error>{
        let PowerState::Cooling(time_stamp) = self else {
            panic!("Invalid state: {:?}", self)
        };

        if time_stamp.elapsed().unwrap() > COOLDOWN_TIME {
            *self = PowerState::PowerOff;
            Ok(None)
        } else {
            Ok(None)
        }
    }

    pub fn process_warming(&mut self, message: &str) -> Result<Option<String>, std::io::Error>{
        let PowerState::Warming(time_stamp) = self else {
            panic!("Invalid state: {:?}", self)
        };

        if time_stamp.elapsed().unwrap() > WARMING_TIME {
            *self = PowerState::LampOn;
            self.process_lamp_on(message)
        } else {
            Ok(None)
        }
    }

    pub fn process_poweroff(&mut self, message: &str) -> Result<Option<String>, std::io::Error>{
        if message == "PWR ON\r" {
            *self = PowerState::Warming(time::SystemTime::now());
            Ok(None)
        } else {
            // Process commands
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid command"))
        }
    }

    pub fn process_lamp_on(&mut self, message: &str) -> Result<Option<String>, std::io::Error> {
        Ok(None)
    }

}

pub struct Epsonlib<'a, T: 'a + Read + Write> {
    // private
    port: &'a mut T,
}

impl<'a, T: 'a + Read + Write> Epsonlib<'a, T> {
    pub fn new(port: &'a mut T) -> Epsonlib<'a, T> {
        Epsonlib {
            // private
            port
        }
    }

    pub fn run_until(&mut self) {
        let mut serial_buf: Vec<u8> = vec![0; 128];
        let mut codec = codec::Codec::new();
        loop {
            match self.port.read(serial_buf.as_mut_slice()) {
                Ok(t) => {
                    if t > 0 {
                        println!("Read {} bytes: {:?}", t, &serial_buf[..t]);
                        match codec.decode(&mut serial_buf[..t]) {
                            Ok(Some(s)) => {
                                println!("Decoded: {}", s);
                                self.print(&s);
                            }
                            Ok(None) => (),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }

    pub fn print(&self, text: &str) {
        println!("Epson: {}", text);
    }
}