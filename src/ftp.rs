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

                let res = ftp.read_message()?;

                match &res[0..1] {
                    "2" => {},
                    _ => {return Err(res)}
                }

                ftp.init_messege = res;
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

        dbg!(&res);

        Ok(res)
    }

    fn send_message(&mut self, msg: String) -> Result<String, String> {
        match self.tcp_control.get_mut().write(msg.as_bytes()) {
            Ok(len) => Ok(format!("Successfully wrote {} bytes", len)),
            Err(e) => Err(format!("Failed to write to the server: {}", e)),
        }
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<String, String> {
        self.send_message("USER jazzcort\r\n".to_string())?;
        let res = self.read_message()?;
        match &res[0..1] {
            "2"|"3" => {},
            _ => {return Err(res)}
        }

        self.send_message(
            "PASS 7a14b3a17a988de5849061a25516f8c5eaf8a16e3202ca966b6b0bfe820d7c01\r\n".to_string(),
        )?;
        let res = self.read_message()?;
        match &res[0..1] {
            "2" => {},
            _ => {return Err(res)}
        }

        self.send_message("TYPE I\r\n".to_string())?;
        let res = self.read_message()?;
        match &res[0..1] {
            "2" => {},
            _ => {return Err(res)}
        }


        self.send_message("MODE S\r\n".to_string())?;
        let res = self.read_message()?;
        match &res[0..1] {
            "2" => {},
            _ => {return Err(res)}
        }

        self.send_message("STRU F\r\n".to_string())?;
        let res = self.read_message()?;
        match &res[0..1] {
            "2" => {},
            _ => {return Err(res)}
        }

        Ok("Successfully logged in".to_string())
    }

    fn read_data_channel(&mut self, mut stream: TcpStream) -> Result<Vec<u8>, String> {
        let mut buf: Vec<u8> = vec![];
        match stream.read_to_end(&mut buf).map_err(|e| format!("{}", e)) {
            Ok(_) => {
                self.read_message()?;
                Ok(buf)
            }
            Err(e) => {
                self.read_message()?;
                Err(e)
            }
        }
    }

    fn write_data_channel(
        &mut self,
        mut stream: TcpStream,
        data: Vec<u8>,
    ) -> Result<String, String> {
        match stream.write(&data).map_err(|e|format!("{}", e)) {
            Ok(len) => {
                stream.shutdown(std::net::Shutdown::Both).map_err(|e| format!("{}", e))?;
                self.read_message()?;
                Ok(format!(
                    "Successfully wrote {} bytes to the data channel",
                    len
                ))
            },
            Err(e) => {
                stream.shutdown(std::net::Shutdown::Both).map_err(|e| format!("{}", e))?;
                self.read_message()?;
                Err(e)
            }
        }
        
        
    }

    pub fn list(&mut self, path: &str) -> Result<String, String> {
        match self.pasv() {
            Ok(stream) => {
                self.send_message(format!("LIST {}\r\n", path))?;
                self.read_message()?;
                let buf: Vec<u8> = self.read_data_channel(stream)?;
                println!("{}", String::from_utf8_lossy(&buf));
                Ok(format!(
                    "successfully read {} directory listing from the server",
                    path
                ))
            }
            Err(e) => Err(e),
        }
    }

    pub fn mkd(&mut self, path: &str) -> Result<String, String> {
        self.send_message(format!("MKD {}\r\n", path))?;
        let res = self.read_message()?;

        match &res[0..1] {
            "2" => Ok(res),
            _ => Err(res),
        }
    }

    pub fn rmd(&mut self, path: &str) -> Result<String, String> {
        self.send_message(format!("RMD {}\r\n", path))?;
        let res = self.read_message()?;

        match &res[0..1] {
            "2" => Ok(res),
            _ => Err(res),
        }
    }

    pub fn dele(&mut self, path: &str) -> Result<String, String> {
        self.send_message(format!("DELE {}\r\n", path))?;
        let res = self.read_message()?;

        match &res[0..1] {
            "2" => Ok(res),
            _ => Err(res),
        }
    }

    pub fn stor(&mut self, file_path: &str, server_path: &str) -> Result<String, String> {
        let mut f = File::open(file_path)
            .map_err(|_e| format!("can't find the local file wiht given path {}", file_path))?;
        let mut buf: Vec<u8> = vec![];
        f.read_to_end(&mut buf).map_err(|e| format!("{}", e))?;
        let stream = self.pasv()?;
        self.send_message(format!("STOR {}\r\n", server_path))?;
        
        let res = self.read_message()?;
        match &res[0..1] {
            "1" => {},
            _ => {return Err(res)}
        }

        self.write_data_channel(stream, buf)?;

        Ok(format!(
            "Successfully transfered {} to {}",
            file_path, server_path
        ))
    }

    pub fn retr(&mut self, file_path: &str, server_path: &str) -> Result<String, String> {
        let stream = self.pasv()?;
        self.send_message(format!("RETR {}\r\n", server_path))?;
        let res = self.read_message()?;
        match &res[0..1] {
            "1" => {},
            _ => {return Err(res)}
        }

        let buf = self.read_data_channel(stream)?;
        let mut f = File::create(file_path).map_err(|e| format!("{}", e))?;
        f.write(&buf).map_err(|e| format!("{}", e))?;

        Ok(format!("Successfully transfered {} to {}", server_path, file_path))
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

        match &ip[0..1] {
            "2" => {},
            _ => {return Err(ip)}
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
            None => return Err(format!("Didn't capture the IP address {}", ip)),
        }
    }
}
