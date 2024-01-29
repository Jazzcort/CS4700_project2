move: complie
	mv ./target/release/project2-FTP ./4700ftp

complie: client
	~/.cargo/bin/cargo build -r

# Thanks for Luke Jianu
client: 
	curl https://sh.rustup.rs -sSf | sh -s -- -y \
	&& ~/.cargo/bin/rustup install --profile=minimal 1.75.0 \
	&& ~/.cargo/bin/rustup default 1.75.0 