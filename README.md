# remote-desktop

### Setup
Install Rust
```sh
curl https://sh.rustup.rs -sSf | sh
source ~/.cargo/env
```
Install Library
```sh
sudo apt install libxdo-dev
sudo apt install libgtk-3-dev
```

### How to run
1. Run Server  
Specify server address and port and frame rate
```sh
cargo run --bin server 192.168.0.21:8888 30
```
2. Run Client  
Specify server address and port
```sh
cargo run --bin client 192.168.0.21:8888
```