extern crate nix;
extern crate mio;

use std::str;
use mio::{Reactor, Handler, IoReader, IoWriter, TcpSocket, SockAddr};

/*
struct Proxy;

impl Handler for Proxy {
    fn accept(token: Token) -> Option<Token> {
        // foo
    }

    fn readable(token);
    fn writable(token);
    fn error(token);
}
*/

struct MyHandler {
    sock: TcpSocket,
    done: bool
}

impl MyHandler {
    pub fn new(sock: TcpSocket) -> MyHandler {
        MyHandler {
            sock: sock,
            done: false
        }
    }
}

impl Handler<uint> for MyHandler {
    fn readable(&mut self, reactor: &mut Reactor, tok: uint) {
        let mut buf = Vec::from_fn(1024, |_| 0);
        let mut i = 0u;

        loop {
            i += 1;

            match self.sock.read(buf.as_mut_slice()) {
                Ok(cnt) => {
                    println!("{}", str::from_utf8(buf.as_slice().slice_to(cnt)));
                }
                Err(e) if e.is_eof() => {
                    println!("EOF");
                    return;
                }
                e => {
                    println!("error: {}", e);
                    return;
                }
            }
        }
    }

    fn writable(&mut self, reactor: &mut Reactor, tok: uint) {
        if self.done {
            return;
        }

        println!("Connected, writing payload");

        self.done = true;
        self.sock.write(b"ZOMG FOO BAR \r\n\r\n").unwrap();
    }
}

pub fn main() {
    println!(" * Initializing reactor");
    let mut reactor = Reactor::<()>::new().unwrap();

    println!(" * Parsing socket address");
    let addr = SockAddr::parse("127.0.0.1:9292").expect("could not parse InetAddr");

    println!(" * Creating socket");
    let sock = TcpSocket::v4().unwrap();

    // Configure options

    println!("Connect socket");
    reactor.connect(sock, &addr, 123u).unwrap();

    println!("Start reactor");
    reactor.run(MyHandler::new(sock));

    /*

    // set sock options

    // reactor.connect();
    reactor.run(MyHandler);

    let proxy = Proxy::new();

    let sock = TcpSocket::v4();
    let reactor = Reactor::new();

    reactor.connect(sock, 1);
    reactor.run();
    */
}
