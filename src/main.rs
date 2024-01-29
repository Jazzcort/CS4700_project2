use clap::{Parser, ValueEnum};
use regex::Regex;
use std::fs;

use ftp::FtpStream;
mod ftp;

#[macro_use]
extern crate lazy_static;

#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Cli {
    /// The operation to execute.
    #[arg(value_enum)]
    operation: Operation,
    /// Parameters for the given operation. This parameter is mandatory.
    param1: String,
    /// Parameters for the given operation. This parameter is only mandatory when using 'cp' or 'mv'
    param2: Option<String>,

    /// Print all messages to and from the FTP server
    #[arg(short, long)]
    verbose:  bool
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Operation {
    Ls,
    Mkdir,
    Rm,
    Rmdir,
    Cp,
    Mv
}

lazy_static! {
    static ref REGEX_USER: Regex = Regex::new(r"ftp://([^:]+)(:.+)?@([^:/]+)(:\d+)?/(.*)").unwrap();
    static ref REGEX_ANONYMOUS: Regex = Regex::new(r"ftp://([^:@/]+)(:\d+)?/([^@]*)").unwrap();
}

fn extract_param(param: &str) -> Result<(&str, &str, &str, &str, &str), String> {

    if REGEX_USER.is_match(param) {
        let cap = REGEX_USER.captures(param).unwrap();

        let username = cap.get(1).unwrap().as_str();
        let password = match cap.get(2) {
            Some(p) => &p.as_str()[1..],
            None => ""
        };
        let host = cap.get(3).unwrap().as_str();
        let port = match cap.get(4) {
            Some(p) => &p.as_str()[1..],
            None => ""
        };
        let path = cap.get(5).unwrap().as_str();

        Ok((username, password, host, port, path))
    } else if REGEX_ANONYMOUS.is_match(param) {
        let cap = REGEX_ANONYMOUS.captures(param).unwrap();
        let username = "anonymous";
        let password = "";
        let host = cap.get(1).unwrap().as_str();
        let port =  match cap.get(2) {
            Some(p) => &p.as_str()[1..],
            None => ""
        };
        let path = cap.get(3).unwrap().as_str();

        Ok((username, password, host, port, path))
    } else {
        Err("Given URL is invalid".to_string())
    }
}

fn main() -> Result<(), String> {

    let cli = Cli::parse();
    // dbg!(cli.operation == Operation::Ls);
    dbg!(&cli);
    // let a = REGEX_USER.captures(&cli.param1);
    // dbg!(a);

    let (username, password, host, port, path) = match extract_param(&cli.param1) {
        Ok(x) => x,
        Err(e) => return Err(e)
    };

    dbg!(&username, &password, &host, &port, &path);

    match &cli.operation {
        Operation::Ls | Operation::Mkdir | Operation::Rm | Operation::Rmdir => {
            let (username, password, host, port, path) = match extract_param(&cli.param1) {
                Ok(x) => x,
                Err(e) => {return Err(e)}
            };

            let mut ftp = match FtpStream::new(host, if !port.is_empty() {port} else {"21"}, cli.verbose) {
                Ok(stream) => stream,
                Err(e) => {
                    return Err(e);
                }
            };

            ftp.login(username, password)?;

            match &cli.operation {
                Operation::Ls => {
                    ftp.list(path)?;
                },
                Operation::Mkdir => {
                    ftp.mkd(path)?;
                },
                Operation::Rm => {
                    ftp.dele(path)?;
                },
                Operation::Rmdir => {
                    ftp.rmd(path)?;
                }
                _ => {}
            }

            Ok(())
        },
        _ => {
            dbg!(&cli.param2);

            match &cli.param2 {
                Some(p) => {
                    let r1 = extract_param(&cli.param1);
                    let r2 = extract_param(&p);

                    match (r1, r2){
                        // From server
                        (Ok((username, password, host, port, path)), Err(_)) => {
                
                            let mut ftp = match FtpStream::new(host, if !port.is_empty() {port} else {"21"}, cli.verbose) {
                                Ok(stream) => stream,
                                Err(e) => {
                                    return Err(e);
                                }
                            };

                            ftp.login(username, password)?;

                            match &cli.operation {
                                Operation::Cp => {
                                    ftp.retr(&p, path)?;
                                },
                                Operation::Mv => {
                                    ftp.retr(&p, path)?;
                                    if let Err(_) = ftp.dele(path) {
                                        fs::remove_file(&p).map_err(|e| format!("{}", e))?;
                                    }
                                }
                                _ => {}
                            }
                        },
                        // To server
                        (Err(_), Ok((username, password, host, port, path))) => {
                            let mut ftp = match FtpStream::new(host, if !port.is_empty() {port} else {"21"}, cli.verbose) {
                                Ok(stream) => stream,
                                Err(e) => {
                                    return Err(e);
                                }
                            };

                            ftp.login(username, password)?;

                            match &cli.operation {
                                Operation::Cp => {
                                    ftp.stor(&cli.param1, path)?;
                                },
                                Operation::Mv => {
                                    ftp.stor(&cli.param1, path)?;
                                    fs::remove_file(&cli.param1).map_err(|e| format!("{}", e))?;
                                },
                                _ => {}
                            }

                        },
                        _ => {return Err("If ARG1 is a local file, then ARG2 must be a URL, and vice-versa.".to_string());}

                    }
                    
                },
                None => {return Err("Didn't provide the second argument for \'cp\' or \'mv\' command".to_string());}
            }

            Ok(())
        }
    }



    // let mut ftp = match FtpStream::new("ftp.4700.network", "21") {
    //     Ok(stream) => stream,
    //     Err(e) => {
    //         writeln!(&mut std::io::stderr(), "{}", e).unwrap();
    //         return Err(e);
    //     }
    // };

    // dbg!(&ftp.init_messege);
    // ftp.login("jazzcort", "7a14b3a17a988de5849061a25516f8c5eaf8a16e3202ca966b6b0bfe820d7c01");
    // dbg!(&ftp.list("./"));

    // dbg!(&ftp.list("./"));
    // dbg!(&ftp.list("./"));
    // dbg!(&ftp.mkd("./test3"));
    // dbg!(&ftp.list("./"));
    // dbg!(&ftp.stor("./test.txt", "./test3/test.txt"));
    // dbg!(&ftp.retr("test11.txt", "./test/testing.txt"));
    // dbg!(&ftp.stor("./testing.txt", "./test1/testing.txt"));

    // dbg!(&ftp.list("./test"));
    // dbg!(&ftp.rmd("./test"));
    // dbg!(&ftp.read_message());
    // let mut buf = [0u8; 5];
    // ftp.tcp_control.read(&mut buf).unwrap();
    // dbg!(String::from_utf8_lossy(&buf));

    

    // Ok(())
}
