use clap::{Parser, ValueEnum};
use regex::Regex;
use std::fs;

use ftp::FtpStream;
mod ftp;

#[macro_use]
extern crate lazy_static;

/**
 * This struct is used to extract the command line arguments.
 */
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

/**
 * This enum is used to validate and classify the operation field of the Cli struct.
 */
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Operation {
    Ls,
    Mkdir,
    Rm,
    Rmdir,
    Cp,
    Mv
}

// Allocate static memory for REGEX_USER and REGEX_ANONYMOUS
lazy_static! {
    static ref REGEX_USER: Regex = Regex::new(r"ftp://([^:]+)(:.+)?@([^:/]+)(:\d+)?/(.*)").unwrap();
    static ref REGEX_ANONYMOUS: Regex = Regex::new(r"ftp://([^:@/]+)(:\d+)?/([^@]*)").unwrap();
}

/**
 * This function is to extract username, password, host, port, and path from
 * the given URL.
 */
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

/**
 * This is the main driver of this CLI.
 */
fn main() -> Result<(), String> {

    // Parse the command line argument
    let cli = Cli::parse();

    // Check the operation kind
    match &cli.operation {
        Operation::Ls | Operation::Mkdir | Operation::Rm | Operation::Rmdir => {
            // Extract the parameters that would be used to create the FtpStream and login to the Ftp server
            let (username, password, host, port, path) = match extract_param(&cli.param1) {
                Ok(x) => x,
                Err(e) => {return Err(e)}
            };

            // Create the control channel with the ftp server
            let mut ftp = match FtpStream::new(host, if !port.is_empty() {port} else {"21"}, cli.verbose) {
                Ok(stream) => stream,
                Err(e) => {
                    return Err(e);
                }
            };

            // Login to Ftp server
            ftp.login(username, password)?;

            // Execute desired command
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
            // Check if the second parameter is entered
            match &cli.param2 {
                Some(p) => {
                    let r1 = extract_param(&cli.param1);
                    let r2 = extract_param(&p);

                    match (r1, r2){
                        // From server (param1 is an URL and param2 is a local path)
                        (Ok((username, password, host, port, path)), Err(_)) => {
                
                            // Create the control channel to the Ftp server
                            let mut ftp = match FtpStream::new(host, if !port.is_empty() {port} else {"21"}, cli.verbose) {
                                Ok(stream) => stream,
                                Err(e) => {
                                    return Err(e);
                                }
                            };

                            // Login to the Ftp server
                            ftp.login(username, password)?;

                            match &cli.operation {
                                Operation::Cp => {
                                    ftp.retr(&p, path)?;
                                },
                                Operation::Mv => {
                                    ftp.retr(&p, path)?;
                                    // Remove the local file if the file deletion is not successful on the server side
                                    if let Err(_) = ftp.dele(path) {
                                        fs::remove_file(&p).map_err(|e| format!("{}", e))?;
                                    }
                                }
                                _ => {}
                            }
                        },
                        // To server (param1 is a local path and param2 is an URL)
                        (Err(_), Ok((username, password, host, port, path))) => {
                            // Create the control channel to the Ftp server
                            let mut ftp = match FtpStream::new(host, if !port.is_empty() {port} else {"21"}, cli.verbose) {
                                Ok(stream) => stream,
                                Err(e) => {
                                    return Err(e);
                                }
                            };

                            // Login to the Ftp server
                            ftp.login(username, password)?;

                            match &cli.operation {
                                Operation::Cp => {
                                    ftp.stor(&cli.param1, path)?;
                                },
                                Operation::Mv => {
                                    ftp.stor(&cli.param1, path)?;
                                    // Remove the local file if the file successfully makes its way to the server
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
}
