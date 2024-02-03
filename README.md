# Project 2: FTP Client

## Approach
My project is comprised of two main components. The first part is the FTP module, where I employ `TcpStream`, `std::fs`, and `std::io` to implement the FTP protocol. This module features functions such as `new` (to initialize a FtpStream with a specified host and port number), `login` (for logging in with a provided username and password), and other functions like `list`, `dele`, `mkd`, etc., which are designed to send server commands to the FTP server.

The second part involves utilizing the FTP module to create a Command-Line Interface (CLI). For this segment, I leverage crates such as `clap` and `regex` to facilitate the extraction of command-line arguments and parameters (such as host, username, password, etc.) from the URL format. Drawing on my previous experience with TCP, the integration of the FTP module into the CLI proceeded smoothly.

## Challenge
The most significant challenge I encountered during the project was mastering Regular Expression syntax. Prior to this project, my experience with handling complex string formats was limited, making it initially challenging to comprehend the intricacies of Regular Expression syntax. However, after investing time and effort into understanding its workings, I gained proficiency in using Regular Expressions as a powerful tool for extracting desired information from formatted strings.

## Code Testing
For testing purposes, I run the debug version of my program through the terminal, ensuring that each execution functions as intended when interacting with FileZilla. Additionally, I perform tests with invalid commands to confirm that the Command-Line Interface (CLI) appropriately catches and handles errors.
