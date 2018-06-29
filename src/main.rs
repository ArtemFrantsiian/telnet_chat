extern crate docopt;
extern crate rand;

use docopt::Docopt;

use std::io::{self, Write};
use std::process;

mod error;
mod client;
mod server;
mod message;

const DEFAULT_ADDR: &'static str = "localhost";
const DEFAULT_PORT: u16 = 10001;
const USAGE: &'static str = "
Telnet Chat.

Usage:
    socket [options]
    socket --help
Options:
    --help        Show this message.
    --addr ADDR   IP address.
    --port PORT   Port number.
";

fn main() {
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.parse())
                      .unwrap_or_else(|e| e.exit());
    let addr = args.get_str("--addr");
    let addr = if addr.is_empty() { DEFAULT_ADDR } else { addr };

    let port = args.get_str("--port");
    let port: u16 = if port.is_empty() { DEFAULT_PORT } else { port.parse().unwrap() };

    if let Err(err) = server::run(addr, port) {
        writeln!(io::stderr(), "{}", err);
        process::exit(1);
    }
}