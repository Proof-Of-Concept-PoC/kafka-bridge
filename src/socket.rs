use std::io::{Read, BufRead, BufReader, Write};
use std::net::{Shutdown, TcpStream};
use std::{thread, time};

#[derive(Debug)]
pub(crate) enum Error {
    Write,
    Read,
}

pub(crate) struct Socket {
    host: String,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

pub(crate) fn log(host: &str, message: &str) {
    println!("{}", json::stringify(json::object!{
        "message" => message,
        "client" => "MyClient",
        "host" => host,
    }));
}

/// # Socket
///
/// The user interface for this library.
///
/// ```private
/// use nats_bridge::socket::Socket;
///
/// let host = "pubsub.pubnub.com:80";
/// let socket = Socket::new(host.to_string());
/// ```
impl Socket {
    pub(crate) fn new(host: &str) -> Self {
        let stream = Socket::connect(host);
        Self {
            host: host.into(),
            stream: stream.try_clone().expect("Unable to clone stream"),
            reader: BufReader::new(stream),
        }
    }

    pub(crate) fn log(&mut self, message: &str) {
        log(&self.host, message);
    }

    /// ## Write Data
    ///
    /// Write string data to the stream.
    ///
    /// ```private
    /// use nats_bridge::socketSocket;
    /// let host: "pubsub.pubnub.com:80".into(),
    /// let mut socket = Socket::new(host);
    /// let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
    /// socket.write(request);
    /// ```
    pub(crate) fn write(&mut self, data: &str) -> Result<usize, Error> {
        let result = self.stream.write(data.as_bytes());
        match result {
            Ok(size) => {
                if size > 0 {
                    return Ok(size);
                }
                self.log("No data has been written.");
                Err(Error::Write)
            }
            Err(error) => {
                self.log(&format!("Unwrittable: {}", error));
                self.log(&format!("Disconnected: {}", error));
                Err(Error::Write)
            }
        }
    }

    /// ## Read Line
    ///
    /// Read a line of data from the stream.
    ///
    /// ```private
    /// use nats_bridge::socket::Socket;
    /// let host: "pubsub.pubnub.com:80";
    /// let mut socket = Socket::new(host.into());
    /// let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
    /// socket.write(request);
    /// let line = socket.readln();
    /// ```
    pub(crate) fn readln(&mut self) -> Result<String, Error> {
        let mut line = String::new();
        let result = self.reader.read_line(&mut line);
        let size = result.unwrap_or_else(|_| 0);

        if size == 0 {
            Err(Error::Read)?;
        }

        Ok(line)
    }

    /// ## Read Bytes
    ///
    /// Read specified amount of data from the stream.
    ///
    /// ```private
    /// use nats_bridge::socket::Socket;
    /// 
    /// let host: "pubsub.pubnub.com:80",
    /// let mut socket = Socket::new(host.into());
    /// let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
    /// socket.write(request);
    /// let data = socket.read(30); // read 30 bytes
    /// println!("{}", data);
    /// ```
    pub(crate) fn read(&mut self, bytes: usize) -> Result<String, Error> {
        let mut buffer = vec![0u8; bytes];
        let result = self.reader.read(&mut buffer);

        if result.is_err() {
            Err(Error::Read)?;
        }

        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    /// ## Disconnect
    ///
    /// This will courteously turn off the connection of your socket.
    ///
    /// ```private
    /// use nats_bridge::socket::Socket;
    /// let host: "pubsub.pubnub.com:80",
    /// let mut socket = Socket::new(host.into());
    /// socket.disconnect();
    /// ```
    pub(crate) fn disconnect(&mut self) {
        self.stream.shutdown(Shutdown::Both).unwrap_or_default();
    }

    pub(crate) fn reconnect(&mut self) {
        thread::sleep(time::Duration::new(1, 0));
        let stream = Socket::connect(&self.host);
        self.stream = stream.try_clone().expect("Unable to clone stream");
        self.reader = BufReader::new(stream);
    }

    fn connect(ip_port: &str) -> TcpStream {
        loop {
            // Open connection and send initialization data
            let host: String = ip_port.into();
            let error = match TcpStream::connect(host) {
                Ok(stream) => {
                    log(ip_port, "Connected to host");
                    return stream;
                }
                Err(error) => error,
            };

            // Retry connection until the host becomes available
            log(ip_port, &format!("{}", error));
            thread::sleep(time::Duration::new(1, 0));
        }
    }
}

#[cfg(test)]
mod socket_tests {
    use super::*;
    use json::object;

    #[test]
    fn write_ok() {
        let host = "www.pubnub.com:80".into();
        let mut socket = Socket::new(host);

        let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
        let _ = socket.write(request).expect("data written");
    }

    #[test]
    fn read_ok() {
        let host = "www.pubnub.com:80".into();
        let mut socket = Socket::new(host);

        let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
        socket.write(request).expect("data written");

        let result = socket.readln();
        assert!(result.is_ok());

        let data = result.expect("data");
        assert!(data.len() > 0);

        let result = socket.readln();
        assert!(result.is_ok());

        let data = result.expect("data");
        assert!(data.len() > 0);
    }
}
