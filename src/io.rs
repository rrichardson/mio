use os;
use error::MioResult;
use sock::Socket;

pub trait IoReader {
    fn read(&mut self, buf: &mut [u8]) -> MioResult<uint>;
}

pub trait IoWriter {
    fn write(&mut self, buf: &[u8]) -> MioResult<uint>;
}

impl<S: Socket> IoReader for S {
    fn read(&mut self, buf: &mut [u8]) -> MioResult<uint> {
        os::read(self.desc(), buf)
    }
}

impl<S: Socket> IoWriter for S {
    fn write(&mut self, buf: &[u8]) -> MioResult<uint> {
        os::write(self.desc(), buf)
    }
}
