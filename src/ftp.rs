use std::{
    fs::File,
    io::{BufReader, Read, Write},
    net::TcpStream,
};

use regex::Regex;

// Allocate a static memory for PORT_REGEX
lazy_static! {
    static ref PORT_REGEX: Regex = Regex::new(r"\((\d+),(\d+),(\d+),(\d+),(\d+),(\d+)\)").unwrap();
}

/**
 * This is the FtpStream struct. It contains the control channel to the Ftp server
 * which is wrapped in BufReader, initial message send by the Ftp server when the control
 * channel is connected successfully, and a bool to indecate whether it should print the
 * server message or not. 
 */
pub struct FtpStream {
    tcp_control: BufReader<TcpStream>,
    init_messege: String,
    verbose_mode: bool
}

// All the functions implemented for FtpStream
impl FtpStream {
    /**
     * This function is to initialize a FtpStream with given host and port number.
     * hostname: The hostname of the Ftp server.
     * port_num: The port number to use.
     * v: To print server message or not.
     * Return Ok(FtpStream) if no error occurs, otherwise Err(String)
     */
    pub fn new(hostname: &str, port_num: &str, v:bool) -> Result<Self, String> {
        TcpStream::connect(format!("{}:{}", hostname, port_num))
            .map_err(|e| format!("connection failed: {}", e))
            .and_then(|stream| {
                // Create the FtpStream instance
                let mut ftp = FtpStream {
                    tcp_control: BufReader::new(stream),
                    init_messege: String::new(),
                    verbose_mode: v
                };

                // Read the initial message
                let res = ftp.read_message()?;

                // Check if the initial connection is successful
                match &res[0..1] {
                    "2" => {},
                    _ => {return Err(res)}
                }

                ftp.init_messege = res;
                Ok(ftp)
            })
    }

    /**
     * This function is to read the message sent by Ftp server.
     * This function can only be used inside the modeul.
     * Return Ok(String) with server message if no error occurs, 
     * otherwise Err(String) with error message
     */
    fn read_message(&mut self) -> Result<String, String> {
        // Buffer array
        let mut buf = [0u8; 16];
        // Where the server message goes
        let mut res = String::new();

        // Continuously read from the server until \r\n appears
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

            // Translate message from utf8 to string
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

        // Print the message to stdout if it's in verbose mode
        if self.verbose_mode {
            println!("{}", &res);
        }

        Ok(res)
    }

    /**
     * This function is to send the command to the Ftp server.
     * This function can only be used inside the module.
     * msg: The command that needs to be sent to the Ftp server.
     * Return Ok(String) with success message if no error occurs, 
     * otherwise Err(String) with error message
     */
    fn send_message(&mut self, msg: String) -> Result<String, String> {
        match self.tcp_control.get_mut().write(msg.as_bytes()) {
            Ok(len) => Ok(format!("Successfully wrote {} bytes", len)),
            Err(e) => Err(format!("Failed to write to the server: {}", e)),
        }
    }

    /**
     * This function is to login the Ftp server with given username and password.
     * username: The username of the cilent.
     * password: The passwrod of the client.
     * Return Ok(String) with success message if no error occurs,
     * otherwise, Err(String)
     */
    pub fn login(&mut self, username: &str, password: &str) -> Result<String, String> {
        self.send_message(format!("USER {}\r\n", username))?;
        // Read the server's response
        let res = self.read_message()?;
        // Check if the server responds the correct code
        match &res[0..1] {
            "2"|"3" => {},
            _ => {return Err(res)}
        }

        // Send password to the server
        self.send_message(
            format!("PASS {}\r\n", password),
        )?;
        let res = self.read_message()?;
        match &res[0..1] {
            "2" => {},
            _ => {return Err(res)}
        }

        // Configure the server to Binary mode
        self.send_message("TYPE I\r\n".to_string())?;
        let res = self.read_message()?;
        match &res[0..1] {
            "2" => {},
            _ => {return Err(res)}
        }


        // Configure the server to Stream mode
        self.send_message("MODE S\r\n".to_string())?;
        let res = self.read_message()?;
        match &res[0..1] {
            "2" => {},
            _ => {return Err(res)}
        }

        // Configure the server to File-Oriented mode
        self.send_message("STRU F\r\n".to_string())?;
        let res = self.read_message()?;
        match &res[0..1] {
            "2" => {},
            _ => {return Err(res)}
        }

        Ok("Successfully logged in".to_string())
    }

    /**
     * This function is to receive data from data channel.
     * This functon can only be used inside the module.
     * stream: The TcpStream of the data channel.
     * Return Ok(Vec<u8>) which contains the file data if no error occurs,
     * otherwise, Err(String) with error message.
     */
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

    /**
     * This function is to write data to the data channel.
     * This function can only be used inside the module.
     * stream: The TcpStream of the data channel.
     * data: The data to be written to the datachannel.
     * Return Ok(String) with success message if no error occurs,
     * otherwise, Err(String) with error message.
     */
    fn write_data_channel(
        &mut self,
        mut stream: TcpStream,
        data: Vec<u8>,
    ) -> Result<String, String> {
        match stream.write(&data).map_err(|e|format!("{}", e)) {
            Ok(len) => {
                // Shutdown data channel to notify the server that the transaction is completed
                stream.shutdown(std::net::Shutdown::Both).map_err(|e| format!("{}", e))?;
                self.read_message()?;
                Ok(format!(
                    "Successfully wrote {} bytes to the data channel",
                    len
                ))
            },
            Err(e) => {
                // Shutdown data channel to notify the server that the transaction is completed
                stream.shutdown(std::net::Shutdown::Both).map_err(|e| format!("{}", e))?;
                self.read_message()?;
                Err(e)
            }
        }
        
        
    }

    /**
     * This function is to perform ls command on the Ftp server. The detail of the
     * directory would be print to the stdout.
     * path: The path of the directory that ls command would be executed.
     * Return Ok(String) with success message if no error occurs,
     * otherwise, Err(String) with error message.
     */
    pub fn list(&mut self, path: &str) -> Result<String, String> {
        // Request data channel
        match self.pasv() {
            Ok(stream) => {
                self.send_message(format!("LIST {}\r\n", path))?;
                let res = self.read_message()?;

                // Check if server response is correct for moving on to next step
                match &res[0..1] {
                    "1" => {},
                    _ => {return Err(res)}
                }

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

    /**
     * This function is to perform mkdir command on the Ftp server.
     * path: The path of the directory to be created.
     * Return Ok(String) with success message if no error occurs,
     * otherwise, Err(String) with error message.
     */
    pub fn mkd(&mut self, path: &str) -> Result<String, String> {
        self.send_message(format!("MKD {}\r\n", path))?;
        let res = self.read_message()?;

        match &res[0..1] {
            "2" => Ok(res),
            _ => Err(res),
        }
    }

    /**
     * This function is to perform rmdir command on the Ftp server.
     * path: The path of the directory to be removed.
     * Return Ok(String) with success message if no error occurs,
     * otherwise, Err(String) with error message.
     */
    pub fn rmd(&mut self, path: &str) -> Result<String, String> {
        self.send_message(format!("RMD {}\r\n", path))?;
        let res = self.read_message()?;

        match &res[0..1] {
            "2" => Ok(res),
            _ => Err(res),
        }
    }

    /**
     * This function is to perform rm command on the Ftp server.
     * path: The path of the file to be removed.
     * Return Ok(String) with success message if no error occurs,
     * otherwise, Err(String) with error message.
     */
    pub fn dele(&mut self, path: &str) -> Result<String, String> {
        self.send_message(format!("DELE {}\r\n", path))?;
        let res = self.read_message()?;

        match &res[0..1] {
            "2" => Ok(res),
            _ => Err(res),
        }
    }

    /**
     * This function is to transfer a given file to the Ftp server.
     * file_path: The path of the file in the local storage.
     * server_path: The path of the file that the file would be stored at after the execution.
     * Return Ok(String) with success message if no error occurs,
     * otherwise, Err(String) with error message.
     */
    pub fn stor(&mut self, file_path: &str, server_path: &str) -> Result<String, String> {
        // Read the local file
        let mut f = File::open(file_path)
            .map_err(|_e| format!("can't find the local file wiht given path {}", file_path))?;
        let mut buf: Vec<u8> = vec![];
        // Write the file data to the u8 vector
        f.read_to_end(&mut buf).map_err(|e| format!("{}", e))?;
        // Request the data channel
        let stream = self.pasv()?;
        self.send_message(format!("STOR {}\r\n", server_path))?;
        
        let res = self.read_message()?;
        // Check if it's legit to send data
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

    /**
     * This function is to transfer a file from the Ftp server to the local storage.
     * file_path: The path of the file that the file would be stored at after the execution.
     * server_path: The path of the file in the server.
     * Return Ok(String) with success message if no error occurs,
     * otherwise, Err(String) with error message.
     */
    pub fn retr(&mut self, file_path: &str, server_path: &str) -> Result<String, String> {
        // Request data channel
        let stream = self.pasv()?;
        self.send_message(format!("RETR {}\r\n", server_path))?;
        let res = self.read_message()?;
        // Check if it's legit to receive data
        match &res[0..1] {
            "1" => {},
            _ => {return Err(res)}
        }

        let buf = self.read_data_channel(stream)?;
        // Create local file
        let mut f = File::create(file_path).map_err(|e| format!("{}", e))?;
        // Write the data to the local file
        f.write(&buf).map_err(|e| format!("{}", e))?;

        Ok(format!("Successfully transfered {} to {}", server_path, file_path))
    }

    /**
     * This function is to request a data channel.
     * This function can only be used inside the module.
     * Return Ok(TcpStream) if no error occurs,
     * otherwise, Err(String) with error message.
     */
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
                // Capture the response containing the ip address of data channel
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

        // Extract the ip address
        match PORT_REGEX.captures(&ip) {
            Some(cap) => {
                // Ip address
                let ip1: u8 = cap.get(1).unwrap().as_str().parse().unwrap();
                let ip2: u8 = cap.get(2).unwrap().as_str().parse().unwrap();
                let ip3: u8 = cap.get(3).unwrap().as_str().parse().unwrap();
                let ip4: u8 = cap.get(4).unwrap().as_str().parse().unwrap();

                // Port number
                let ip5: u16 = cap.get(5).unwrap().as_str().parse().unwrap();
                let ip6: u16 = cap.get(6).unwrap().as_str().parse().unwrap();

                // Transfer the port number into decimal format
                let port = (ip5 << 8) + ip6;

                return TcpStream::connect(format!("{}.{}.{}.{}:{}", ip1, ip2, ip3, ip4, port))
                    .map_err(|e| format!("can't connect to file stream at {},  error: {}", ip, e));
            }
            None => return Err(format!("Didn't capture the IP address {}", ip)),
        }
    }
}
