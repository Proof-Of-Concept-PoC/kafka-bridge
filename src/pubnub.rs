// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Libs
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// PubNub
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct PubNub {
    pubkey: String,
    subkey: String,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl PubNub {
    pub fn new(host: String, pubkey: String, subkey: String) -> Result<PubNub, std::io::Error> {
        let mut stream = TcpStream::connect(host).unwrap();
        Ok(PubNub {
            pubkey: pubkey.clone(),
            subkey: subkey.clone(),
            stream: stream.try_clone().unwrap(),
            reader: BufReader::new(stream),
        })
    }

    pub fn publish(&mut self, channel: String, message: String) -> Result<(), std::io::Error> {
        let uri = format!(
            "/publish/{}/{}/0/{}/0/{}",
            self.pubkey, self.subkey, channel, message
        );

        let request = format!("GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n", uri);
        let _ = self.stream.write(request.as_bytes());

        loop {
            let mut buf = String::new();
            let _count = self.reader.read_line(&mut buf).unwrap();
            if _count == 2 {
                break;
            }
            println!("{}: {}", _count.to_string(), buf.to_string());
        }

        Ok(())
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Tests
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_ok() {
        let result = PubNub::new(
            "psdsn.pubnub.com:80".to_string(),
            "demo".to_string(),
            "demo".to_string(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn publish_ok() {
        let result = PubNub::new(
            "psdsn.pubnub.com:80".to_string(),
            "demo".to_string(),
            "demo".to_string(),
        );
        assert!(result.is_ok());

        let mut pubnub = result.unwrap();
        let result = pubnub.publish("demo".to_string(), "123".to_string());
        assert!(result.is_ok());
    }
}
