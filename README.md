# elmo-api-rs

This is a Rust API for elmo (elegant live monitoring of Oscar). It is a work in progress.

## Dependencies
Rust compiler and Cargo package manager, which can be installed using the command below.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Usage
To start the server, run the command below in the terminal.
```bash
cargo run   
```

Then in another terminal, you can make a request to the server using the command below.
```bash
curl http://localhost:3000/cpu
```



