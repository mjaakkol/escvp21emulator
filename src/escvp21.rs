use std::io::{Read, Write, Error, ErrorKind};
use bytes::BytesMut;


use crate::commands::CommandProcessor;


pub struct Codec {
    // private
    buffer: BytesMut,
}

impl Codec {
    pub fn new() -> Codec {
        Codec {
            // private
            buffer: BytesMut::with_capacity(128),
        }
    }

    pub fn decode(&mut self, src: &[u8]) -> Result<Option<String>, Error> {
        self.buffer.extend_from_slice(src);
        let newline = self.buffer.as_ref().iter().position(|b| *b == b'\r');
        if let Some(n) = newline {
            let mut line = self.buffer.split_to(n + 1);
            line.resize(line.len() - 1, 0); // Removing the training \r
            let str_result = match std::str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(Error::new(ErrorKind::Other, "Invalid String")),
            };
            self.buffer.clear();
            return str_result;
        }
        Ok(None)
    }

}

pub fn start<T: Read + Write>(mut port: T, warming: u32, cooling: u32) {
    let mut serial_buf: Vec<u8> = vec![0; 128];
    let mut codec = Codec::new();

    let mut processor = CommandProcessor::new(warming as u64, cooling as u64);
    loop {
        match port.read(serial_buf.as_mut_slice()) {
            Ok(t) => {
                if t > 0 {
                    //println!("Read {} bytes: {:?}", t, &serial_buf[..t]);

                    match codec.decode(&serial_buf[..t]) {
                        Ok(Some(s)) => {
                            println!("Decoded: {:?}", s);
                            match processor.process_message(&s) {
                                Ok(Some(output)) => {
                                    println!("Output: {output}");
                                    port.write(output.as_bytes()).unwrap();
                                },
                                Ok(None) => (),
                                Err(e) => {
                                    eprintln!("Projector error {:?} for command {s}", e);
                                    port.write(b"ERR").unwrap();
                                },
                            }
                            port.write(b"\r:").unwrap();
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


#[cfg(test)]
mod tests {
    use super::*;
    //use serialport::TTYPort;

    use std::sync::mpsc::{channel, Sender, Receiver};

    struct VirtualPort{
        receiver: Receiver<Vec<u8>>,
        sender: Sender<Vec<u8>>,
    }

    impl VirtualPort {
        fn pair() -> (VirtualPort, VirtualPort) {
            let (sender1, receiver1) = channel();
            let (sender2, receiver2) = channel();
            (VirtualPort { receiver: receiver1, sender: sender2 }, VirtualPort { receiver: receiver2, sender: sender1 })
        }
    }

    impl Read for VirtualPort {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
            let data = self.receiver.recv().unwrap();
            let len = data.len();
            buf[..len].copy_from_slice(&data);
            Ok(len)
        }
    }

    impl Write for VirtualPort {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
            self.sender.send(buf.to_vec()).unwrap();
            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<(), Error> {
            Ok(())
        }
    }


    #[test]
    fn test_transaction() {
        let (mut master, mut slave) = VirtualPort::pair(); //.unwrap();

        std::thread::spawn(move || {
            start(&mut slave,2, 1);
        });

        master.write(b"SNO?\r").unwrap();

        let mut buf: Vec<u8> = vec![0; 128];
        let t = master.read(buf.as_mut_slice()).unwrap();
        let output = String::from_utf8(buf[..t].to_vec()).unwrap();
        assert_eq!(output, "SNO=1234567890");

        let t = master.read(buf.as_mut_slice()).unwrap();
        let output = String::from_utf8(buf[..t].to_vec()).unwrap();
        assert_eq!(output, "\r:");

        // Testing error case
        master.write(b"SNO 1234567890\r").unwrap();

        let t = master.read(buf.as_mut_slice()).unwrap();
        //println!("Read {} bytes: {:?}", t, &buf[..t]);
        let output = String::from_utf8(buf[..t].to_vec()).unwrap();
        assert_eq!(output, "ERR");

        let t = master.read(buf.as_mut_slice()).unwrap();
        let output = String::from_utf8(buf[..t].to_vec()).unwrap();
        assert_eq!(output, "\r:");
    }
}