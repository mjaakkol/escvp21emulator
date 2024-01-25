use std::io::{Read, Write};

use crate::{
    codec,
    commands::CommandProcessor
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
                        //println!("Read {} bytes: {:?}", t, &serial_buf[..t]);

                        match codec.decode(&serial_buf[..t]) {
                            Ok(Some(s)) => {
                                println!("Decoded: {:?}", s);
                                match processor.process_message(&s) {
                                    Ok(Some(output)) => {
                                        println!("Output: {output}");
                                        self.port.write(output.as_bytes()).unwrap();
                                    },
                                    Ok(None) => (),
                                    Err(e) => {
                                        eprintln!("Projector error {:?} for command {s}", e);
                                        self.port.write(b"ERR").unwrap();
                                    },
                                }
                                self.port.write(b"\r:").unwrap();
                            }
                            Ok(None) => (),
                            Err(e) => eprintln!("Error: {:?}", e),
                        };
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    eprintln!("timeout");
                },
                Err(e) => {
                    eprintln!("{:?}", e);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serialport::TTYPort;

    #[test]
    fn test_transaction() {
        let (mut master, mut slave) = TTYPort::pair().unwrap();

        std::thread::spawn(move || {
            let mut epson = Epsonlib::new(&mut slave);
            epson.run_until();
        });

        master.write(b"SNO?\r").unwrap();

        let mut buf: Vec<u8> = vec![0; 128];
        let t = master.read(buf.as_mut_slice()).unwrap();
        let output = String::from_utf8(buf[..t].to_vec()).unwrap();
        assert_eq!(output, "1234567890\r:");

        // Testing error case
        master.write(b"SNO 1234567890\r").unwrap();

        let t = master.read(buf.as_mut_slice()).unwrap();
        println!("Read {} bytes: {:?}", t, &buf[..t]);
        let output = String::from_utf8(buf[..t].to_vec()).unwrap();
        assert_eq!(output, "ERR\r:");
    }
}