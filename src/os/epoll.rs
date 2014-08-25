use std::mem;
use nix::fcntl::Fd;
use nix::sys::epoll::*;
use error::{MioResult, MioError};
use os::IoDesc;
use reactor::{IoEvent, IoEventKind, IoReadable, IoWritable, IoError};

pub struct Selector {
    epfd: Fd
}

impl Selector {
    pub fn new() -> MioResult<Selector> {
        let epfd = try!(epoll_create().map_err(MioError::from_sys_error));

        Ok(Selector { epfd: epfd })
    }

    /// Wait for events from the OS
    pub fn select(&mut self, evts: &mut Events, timeout_ms: uint) -> MioResult<()> {
        // Wait for epoll events for at most timeout_ms milliseconds
        let cnt = try!(epoll_wait(self.epfd, evts.events.as_mut_slice(), timeout_ms)
                           .map_err(MioError::from_sys_error));

        evts.len = cnt;
        Ok(())
    }

    /// Register event interests for the given IO handle with the OS
    pub fn register(&mut self, io: IoDesc, token: u64) -> MioResult<()> {
        let interests = EPOLLIN | EPOLLOUT | EPOLLERR;

        let info = EpollEvent {
            events: interests | EPOLLET,
            data: token
        };

        epoll_ctl(self.epfd, EpollCtlAdd, io.fd, &info)
            .map_err(MioError::from_sys_error)
    }
}

pub struct Events {
    len: uint,
    events: [EpollEvent, ..1024]
}

impl Events {
    pub fn new() -> Events {
        Events {
            len: 0,
            events: unsafe { mem::uninitialized() }
        }
    }

    #[inline]
    pub fn len(&self) -> uint {
        self.len
    }

    #[inline]
    pub fn get(&self, idx: uint) -> IoEvent {
        if idx >= self.len {
            fail!("invalid index");
        }

        let epoll = self.events[idx].events;
        let mut kind = IoEventKind::empty();

        debug!("epoll event = {}", epoll);

        if epoll.contains(EPOLLIN) {
            kind = kind | IoReadable;
        }

        if epoll.contains(EPOLLOUT) {
            kind = kind | IoWritable;
        }

        // EPOLLHUP - Usually means a socket error happened
        if epoll.contains(EPOLLERR) {
            kind = kind | IoError;
        }

        let token = self.events[idx].data;

        IoEvent::new(kind, token)
    }
}
