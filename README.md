# Project 2: FTP Client

## Approach
My project consists of two parts. The first part is the ftp module. In this module, I use `TcpSteam`, `std::fs`, and `std::io` to implement a FTP protocal. It contains the functions like `new` (Initialize a FtpStream with host and port number), `login` (Login with given username and password), `list`, `dele`, `mkd`...ect (The rest of the functions is used to send server command to the FTP server). Then, the second part is to use the ftp module I implemented to create a CLI. In this part, I use crates like `clap` and `regex` to helper me extract the command line argument and parameters (host, username, password...etc) from the URL format. Due to the Tcp experience I gained from previous project, everything went pretty smoothly.

## Challenge
The biggest challenge I faced during the project is to use the Regular Expression syntax. I hadn't had too many experiences of dealing with complexing string format and it took time to fully understand how the syntax work in Regular expression. However, after I figured it out, it becomes a very useful tool to extract desired infos from a formatted string.

## Code Testing
Fro the teesting, I execute the debug version through my terminal and check if each execution works the way it should be with FileZilla. Also, I test some invalid commands to check the CLI catch the error properly.
