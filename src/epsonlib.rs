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

    pub fn get_state(&self) -> &str {
        match self {
            PowerState::PowerOff => "PowerOff",
            PowerState::Warming(_) => "Warming",
            PowerState::Cooling(_) => "Cooling",
            PowerState::LampOn => "LampOn",
            PowerState::Terminated => "Terminated",
        }
    }

    pub fn process_cooling(&mut self, message: &str) -> Result<Option<String>, std::io::Error>{
        Ok(None)
    }

    pub fn process_warming(&mut self, message: &str) -> Result<Option<String>, std::io::Error>{
        Ok(None)
    }

    pub fn process_poweroff(&mut self, message: &str) -> Result<Option<String>, std::io::Error>{
        Ok(None)
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