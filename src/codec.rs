use std::io;
use bytes::{BufMut, BytesMut};

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

    pub fn decode(&mut self, src: &[u8]) -> Result<Option<String>, io::Error> {
        self.buffer.extend_from_slice(src);
        let newline = self.buffer.as_ref().iter().position(|b| *b == b'\r');
        if let Some(n) = newline {
            let mut line = self.buffer.split_to(n + 1);
            line.resize(line.len() - 1, 0); // Removing the training \r
            let str_result = match std::str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
            self.buffer.clear();
            return str_result;
        }
        Ok(None)
    }

    pub fn encode(&mut self, item: &str, dst: &mut BytesMut) -> Result<(), io::Error> {
        if item.len() > 0 {
            dst.put(format!("{}\r:", item).as_bytes());
        } else {
            dst.put(":".as_bytes());
        }
        Ok(())
    }
}