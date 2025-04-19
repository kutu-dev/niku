<div align="center">
<h1>NIKU</h1>

Send files and folders over P2P with Iroh and a HTTP signaling server.
</div>

> [!WARNING]
> This project is not ready for production usage but still useful for tinker around with it. The source code contains hard coded URLs on constants to domains that have **not** been bought and servers that are **not** up and running.

## Prerequisites
You need the latest stable Rust toolchain installed on your system, follow the [official installation instructions](https://www.rust-lang.org/learn/get-started).

## Running the app
First you need to run the backend server:
```sh
./scripts/backend.sh
```

Then run the client:
```sh
./scripts cli.sh <args>
```

## Acknowledgements
- Created with :heart: by [Jorge "Kutu" Dobón Blanco](https://dobon.dev).
