use error::MioResult;
use handler::{Handler, Token};
use sock::*;
use os;

/// A lightweight IO reactor.
///
/// An internal lookup structure is used to associate tokens with io
/// descriptors as well as track whether a socket is a listener or not.

#[deriving(Clone, Show)]
pub struct ReactorConfig;

pub struct Reactor {
    selector: os::Selector
}

impl<T: Token> Reactor {
    /// Initializes a new reactor. The reactor will not be running yet.
    pub fn new() -> MioResult<Reactor> {
        Ok(Reactor {
            selector: try!(os::Selector::new())
        })
    }

    /// Registers an IO descriptor with the reactor. TODO: Make this public.
    fn register(&mut self, io: os::IoDesc, token: T) -> MioResult<()> {
        debug!("registering IO with reactor");

        // Register interets for this socket
        try!(self.selector.register(io, token.to_u64()));

        Ok(())
    }

    /// Connects the socket to the specified address. When the operation
    /// completes, the handler will be notified with the supplied token.
    pub fn connect<S: Socket>(&mut self, io: S,
                              addr: &SockAddr, token: T) -> MioResult<()> {

        debug!("socket connect; addr={}", addr);

        // Get the IO descriptor
        let desc = io.desc();

        // Attempt establishing the context. This may not complete immediately.
        if try!(os::connect(desc, addr)) {
            // On some OSs, connecting to localhost succeeds immediately. In
            // this case, queue the writable callback for execution during the
            // next reactor tick.
            debug!("socket connected immediately; addr={}", addr);
        }

        // Register interest with socket on the reactor
        try!(self.register(desc, token));

        Ok(())
    }

    /*
    pub fn listen(&mut self, _io: IoHandle, _token: T) {
        unimplemented!()
    }
    */

    pub fn shutdown(&mut self) {
        unimplemented!()
    }

    pub fn run<H: Handler<T>>(&mut self, mut handler: H) {
        // Created here for stack allocation
        let mut events = os::Events::new();
        let run = true;

        while run { // TODO: Have stop condition
            debug!("reactor tick");

            // Check the registered IO handles for any new events
            self.io_poll(&mut events, &mut handler);
        }
    }

    fn io_poll<H: Handler<T>>(&mut self, events: &mut os::Events, handler: &mut H) {
        self.selector.select(events, 1000).unwrap();

        let mut i = 0u;

        while i < events.len() {
            let evt = events.get(i);
            let tok = Token::from_u64(evt.token);

            if evt.is_readable() {
                handler.readable(self, tok);
            }

            if evt.is_writable() {
                handler.writable(self, tok);
            }

            if evt.is_error() {
                println!(" + ERROR");
            }

            i += 1;
        }
    }
}

bitflags!(
    #[deriving(Show)]
    flags IoEventKind: uint {
        static IoReadable = 0x001,
        static IoWritable = 0x002,
        static IoError    = 0x004
    }
)

#[deriving(Show)]
pub struct IoEvent {
    kind: IoEventKind,
    token: u64
}

impl IoEvent {
    pub fn new(kind: IoEventKind, token: u64) -> IoEvent {
        IoEvent {
            kind: kind,
            token: token
        }
    }

    pub fn is_readable(&self) -> bool {
        self.kind.contains(IoReadable)
    }

    pub fn is_writable(&self) -> bool {
        self.kind.contains(IoWritable)
    }

    pub fn is_error(&self) -> bool {
        self.kind.contains(IoError)
    }
}
