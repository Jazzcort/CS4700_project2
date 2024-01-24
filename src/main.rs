use std::io::{Read, Write, BufRead};
use std::net::TcpStream;

use ftp::FtpStream;
mod ftp;

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), String> {

    let mut ftp = match FtpStream::new("ftp.4700.network", "21") {
        Ok(stream) => stream,
        Err(e) => {
            writeln!(&mut std::io::stderr(), "{}", e).unwrap();
            return Err(e);
        }
    };

    dbg!(&ftp.init_messege);
    ftp.login("jazzcort", "7a14b3a17a988de5849061a25516f8c5eaf8a16e3202ca966b6b0bfe820d7c01");
    // let mut buf = [0u8; 5];
    // ftp.tcp_control.read(&mut buf).unwrap();
    // dbg!(String::from_utf8_lossy(&buf));

    Ok(())
}
