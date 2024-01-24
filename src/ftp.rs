use std::{
    fs::File,
    io::{BufReader, Read, Write},
    net::TcpStream,
};

use regex::Regex;

lazy_static! {
    static ref PORT_REGEX: Regex = Regex::new(r"\((\d+),(\d+),(\d+),(\d+),(\d+),(\d+)\)").unwrap();
}

pub struct FtpStream {
    pub tcp_control: BufReader<TcpStream>,
    pub init_messege: String,
}

impl FtpStream {
    pub fn new(hostname: &str, port_num: &str) -> Result<Self, String> {
        TcpStream::connect(format!("{}:{}", hostname, port_num))
            .map_err(|e| format!("connection failed: {}", e))
            .and_then(|stream| {
                let mut ftp = FtpStream {
                    tcp_control: BufReader::new(stream),
                    init_messege: "".to_string(),
                };

                ftp.init_messege = match ftp.read_message() {
                    Ok(msg) => msg,
                    Err(e) => return Err(e),
                };

                Ok(ftp)
            })
    }

    fn read_message(&mut self) -> Result<String, String> {
        let mut buf = [0u8; 16];
        let mut res = String::new();

        loop {
            match self
                .tcp_control
                .read(&mut buf)
                .map_err(|e| format!("can't read the server response: {}", e))
            {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            }

            let tmp = String::from_utf8_lossy(&mut buf).to_string();

            match tmp.find("\r\n") {
                Some(_) => {
                    let (last_part, _) = tmp.split_at(tmp.find("\r\n").unwrap());
                    res.push_str(last_part);
                    break;
                }
                None => {
                    res.push_str(tmp.as_str());
                }
            }
        }

        Ok(res)
    }

    fn send_message(&mut self, msg: String) -> Result<String, String> {
        match self.tcp_control.get_mut().write(msg.as_bytes()) {
            Ok(len) => Ok(format!("Successfully wrote {} letters", len)),
            Err(e) => Err(format!("Failed to write to the server: {}", e)),
        }
    }

    pub fn login(mut self, username: &str, password: &str) -> Result<String, String> {
        self.send_message("USER jazzcort\r\n".to_string());
        dbg!(self.read_message());

        self.send_message(
            "PASS 7a14b3a17a988de5849061a25516f8c5eaf8a16e3202ca966b6b0bfe820d7c01\r\n".to_string(),
        );
        dbg!(self.read_message());

        self.send_message("TYPE I\r\n".to_string());
        dbg!(self.read_message());

        self.send_message("MODE S\r\n".to_string());
        dbg!(self.read_message());

        self.send_message("STRU F\r\n".to_string());
        dbg!(self.read_message());

        // self.send_message("PASV F\r\n".to_string());
        // let ip_message = self.read_message().unwrap();
        // dbg!(&ip_message);
        // let port_re: Regex = Regex::new(r"\((\d+),(\d+),(\d+),(\d+),(\d+),(\d+)\)").unwrap();
        // let cap = port_re.captures(&ip_message).unwrap();

        // let ip1: u8 = cap.get(1).unwrap().as_str().parse().unwrap();
        // let ip2: u8 = cap.get(2).unwrap().as_str().parse().unwrap();
        // let ip3: u8 = cap.get(3).unwrap().as_str().parse().unwrap();
        // let ip4: u8 = cap.get(4).unwrap().as_str().parse().unwrap();

        // let ip5: u16 = cap.get(5).unwrap().as_str().parse().unwrap();
        // let ip6: u16 = cap.get(6).unwrap().as_str().parse().unwrap();

        // let port = (ip5 << 8) + ip6;

        // let mut file_stream =
        //     TcpStream::connect(format!("{}.{}.{}.{}:{}", ip1, ip2, ip3, ip4, port)).unwrap();

        // let mut file_stream = self.pasv().unwrap();

        // self.ls("./");

        // self.send_message("LIST ./\r\n".to_string());
        // dbg!(self.read_message());

        // let mut buf: [u8; 256] = [0; 256];

        // file_stream.read(&mut buf);

        // dbg!(String::from_utf8_lossy(&buf));

        // self.send_message("RETR ./nu-seal.png\r\n".to_string());
        // dbg!(self.read_message());

        // let mut buf: Vec<u8> = vec![];

        // file_stream.read_to_end(&mut buf);

        // let mut f = File::create("./nu-seal.png").unwrap();
        // f.write(&buf);

        // let res = file_stream.shutdown(std::net::Shutdown::Read);
        // dbg!(res);
        // let mut f = File::open("test.txt").unwrap();
        // let mut data: Vec<u8> = vec![];
        // f.read_to_end(&mut data);

        // self.send_message("STOR ./test13.txt\r\n".to_string());
        // dbg!(self.read_message());

        // let a = file_stream.write(&data);

        // dbg!(a);

        // let mut buf: [u8; 256] = [0; 256];

        // file_stream.read(&mut buf);

        // dbg!(String::from_utf8_lossy(&buf));

        dbg!(self.rm("./test2"));

        Ok("rhgieur".to_string())
    }

    fn read_data_channel(mut stream: TcpStream) -> Result<Vec<u8>, String> {
        let mut buf: Vec<u8> = vec![];
        stream.read_to_end(&mut buf).map_err(|e| format!("{}", e))?;
        return Ok(buf);
    }

    pub fn ls(&mut self, path: &str) -> Result<String, String> {
        match self.pasv() {
            Ok(stream) => {
                self.send_message(format!("LIST {}\r\n", path))?;
                let buf: Vec<u8> = FtpStream::read_data_channel(stream)?;
                println!("{}", String::from_utf8_lossy(&buf));
                Ok(format!("successfully read {} directory listing from the server", path))
            }
            Err(e) => Err(e),
        }
    }

    pub fn mkdir(&mut self, path: &str) -> Result<String, String> {
        self.send_message(format!("MKD {}\r\n", path))?;
        let res = self.read_message()?;

        match &res[0..1] {
            "2" => {Ok(res)},
            _ => {Err(res)}
        }
    }

    pub fn rm(&mut self, path: &str) -> Result<String, String> {
        self.send_message(format!("RMD {}\r\n", path))?;
        let res = self.read_message()?;

        match &res[0..1] {
            "2" => {Ok(res)},
            _ => {Err(res)}
        }
    }

    #[allow(unused)]
    fn pasv(&mut self) -> Result<TcpStream, String> {
        match self.send_message("PASV F\r\n".to_string()) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }

        let mut ip = String::new();

        match self.read_message() {
            Ok(ip_address) => {
                ip = ip_address;
            }
            Err(e) => {
                return Err(e);
            }
        }

        match PORT_REGEX.captures(&ip) {
            Some(cap) => {
                let ip1: u8 = cap.get(1).unwrap().as_str().parse().unwrap();
                let ip2: u8 = cap.get(2).unwrap().as_str().parse().unwrap();
                let ip3: u8 = cap.get(3).unwrap().as_str().parse().unwrap();
                let ip4: u8 = cap.get(4).unwrap().as_str().parse().unwrap();

                let ip5: u16 = cap.get(5).unwrap().as_str().parse().unwrap();
                let ip6: u16 = cap.get(6).unwrap().as_str().parse().unwrap();

                let port = (ip5 << 8) + ip6;

                return TcpStream::connect(format!("{}.{}.{}.{}:{}", ip1, ip2, ip3, ip4, port))
                    .map_err(|e| format!("can't connect to file stream at {},  error: {}", ip, e));
            }
            None => return Err(format!("Didn't capture the IP address")),
        }
    }
}
