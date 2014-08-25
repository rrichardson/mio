use error::{MioResult, MioError};
use sock::{AddressFamily, Inet, Inet6, SockAddr, InetAddr, IpV4Addr};

mod nix {
    pub use nix::fcntl::Fd;
    pub use nix::errno::EINPROGRESS;
    pub use nix::sys::socket::*;
    pub use nix::unistd::*;
}

/// Represents the OS's handle to the IO instance. In this case, it is the file
/// descriptor.
#[deriving(Show)]
pub struct IoDesc {
    pub fd: nix::Fd
}

pub fn socket(af: AddressFamily) -> MioResult<IoDesc> {
    let family = match af {
        Inet  => nix::AF_INET,
        Inet6 => nix::AF_INET6,
        _     => unimplemented!()
    };

    Ok(IoDesc {
        fd: try!(nix::socket(family, nix::SOCK_STREAM, nix::SOCK_NONBLOCK | nix::SOCK_CLOEXEC)
                    .map_err(MioError::from_sys_error))
    })
}

pub fn connect(io: IoDesc, addr: &SockAddr) -> MioResult<bool> {
    match nix::connect(io.fd, &from_sockaddr(addr)) {
        Ok(_) => Ok(true),
        Err(e) => {
            match e.kind {
                nix::EINPROGRESS => Ok(false),
                _ => Err(MioError::from_sys_error(e))
            }
        }
    }
}

pub fn bind(io: IoDesc, addr: &SockAddr) -> MioResult<()> {
    nix::bind(io.fd, &from_sockaddr(addr))
        .map_err(MioError::from_sys_error)
}

pub fn listen(io: IoDesc, backlog: uint) -> MioResult<()> {
    nix::listen(io.fd, backlog)
        .map_err(MioError::from_sys_error)
}

pub fn accept(io: IoDesc) -> MioResult<IoDesc> {
    Ok(IoDesc {
        fd: try!(nix::accept4(io.fd, nix::SOCK_NONBLOCK | nix::SOCK_CLOEXEC)
                     .map_err(MioError::from_sys_error))
    })
}

#[inline]
pub fn read(io: IoDesc, dst: &mut [u8]) -> MioResult<uint> {
    let res = try!(nix::read(io.fd, dst).map_err(MioError::from_sys_error));

    if res == 0 {
        return Err(MioError::eof());
    }

    Ok(res)
}

#[inline]
pub fn write(io: IoDesc, src: &[u8]) -> MioResult<uint> {
    nix::write(io.fd, src).map_err(MioError::from_sys_error)
}

fn from_sockaddr(addr: &SockAddr) -> nix::SockAddr {
    use std::mem;

    match *addr {
        InetAddr(ip, port) => {
            match ip {
                IpV4Addr(a, b, c, d) => {
                    let mut addr: nix::sockaddr_in = unsafe { mem::zeroed() };

                    addr.sin_family = nix::AF_INET as nix::sa_family_t;
                    addr.sin_port = port.to_be();
                    addr.sin_addr = ip4_to_inaddr(a, b, c, d);

                    nix::SockIpV4(addr)
                }
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}

fn ip4_to_inaddr(a: u8, b: u8, c: u8, d: u8) -> nix::in_addr {
    let ip = (a as u32 << 24) |
             (b as u32 << 16) |
             (c as u32 <<  8) |
             (d as u32 <<  0);

    nix::in_addr {
        s_addr: Int::from_be(ip)
    }
}
