# Tokio Chat

A simple asynchronous chat room application written in Rust using the Tokio runtime.  
The project is organized as a Cargo workspace with separate **server** and **client** binaries.

Both `server` and `client` are independent Rust binaries and serve as the main entry points for the application.

---

![](/demo.gif)

---

## Getting Started

Clone this repository:

``` sh
$ git clone https://github.com/9bn1dyp/tokio-chat.git
```

Move into dir:

``` sh
$ cd tokio-chat
```

Start server:

``` sh
$ cd server
$ cargo run
```

Then in another terminal:

``` sh
$ cd client
$ cargo run
```

