use std::io;
use std::io::Write;

#[derive(Debug, Default)]
pub struct WriterProxy {
    pub buffer: Vec<u8>,
    pub flushed: bool,
}

impl Write for WriterProxy {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flushed = true;

        Ok(())
    }
}
