use std::fmt;
use std::path::Path;
use std::from_str::FromStr;
use error::MioResult;
use os;

// TODO: A lot of this will most likely get moved into OS specific files

pub use std::io::net::ip::{IpAddr, Port};
pub use std::io::net::ip::Ipv4Addr as IpV4Addr;

// Types of sockets
pub enum AddressFamily {
    Inet,
    Inet6,
    Unix,
}

pub trait Socket {
    fn desc(&self) -> os::IoDesc;
}

pub struct TcpSocket {
    desc: os::IoDesc
}

impl TcpSocket {
    pub fn v4() -> MioResult<TcpSocket> {
        TcpSocket::new(Inet)
    }

    pub fn v6() -> MioResult<TcpSocket> {
        TcpSocket::new(Inet6)
    }

    fn new(family: AddressFamily) -> MioResult<TcpSocket> {
        Ok(TcpSocket { desc: try!(os::socket(family)) })
    }
}

impl Socket for TcpSocket {
    fn desc(&self) -> os::IoDesc {
        self.desc
    }
}

pub struct UnixSocket {
    desc: os::IoDesc
}

impl Socket for UnixSocket {
    fn desc(&self) -> os::IoDesc {
        self.desc
    }
}

pub enum SockAddr {
    UnixAddr(Path),
    InetAddr(IpAddr, Port)
}

impl SockAddr {
    pub fn parse(s: &str) -> Option<SockAddr> {
        use std::io::net::ip;

        let addr: Option<ip::SocketAddr> = FromStr::from_str(s);
        addr.map(|a| InetAddr(a.ip, a.port))
    }
}

impl FromStr for SockAddr {
    fn from_str(s: &str) -> Option<SockAddr> {
        SockAddr::parse(s)
    }
}

impl fmt::Show for SockAddr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InetAddr(ip, port) => write!(fmt, "{}:{}", ip, port),
            _ => write!(fmt, "not implemented")
        }
    }
}
