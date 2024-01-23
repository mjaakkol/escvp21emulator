use std::io::{Read, Write};

use crate::{
    codec,
    commands::CommandProcessor,
};

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

        let mut processor = CommandProcessor::new();
        loop {
            match self.port.read(serial_buf.as_mut_slice()) {
                Ok(t) => {
                    if t > 0 {
                        println!("Read {} bytes: {:?}", t, &serial_buf[..t]);
                        match codec.decode(&mut serial_buf[..t]) {
                            Ok(Some(s)) => {
                                self.print(&s);
                                match processor.process_message(&s) {
                                    Ok(Some(s)) => {
                                        self.port.write(s.as_bytes()).unwrap();
                                    },
                                    Ok(None) => (),
                                    Err(e) => eprintln!("{:?}", e),
                                }
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