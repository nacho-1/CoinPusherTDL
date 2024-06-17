use std::{
    io,
    net::{Shutdown, TcpStream},
    time::Duration,
};

pub trait Close {
    fn close(&mut self) -> io::Result<()>;
}

#[allow(dead_code)]
pub trait TryClone {
    fn try_clone(&self) -> io::Result<Self>
    where
        Self: Sized;
}

pub trait Interrupt {
    fn alert(&mut self, when: Duration) -> io::Result<()>;

    fn sleep(&mut self) -> io::Result<()>;
}

#[allow(dead_code)]
impl TryClone for TcpStream {
    fn try_clone(&self) -> io::Result<Self>
    where
        Self: Sized,
    {
        TcpStream::try_clone(self)
    }
}

impl Interrupt for TcpStream {
    #[inline(always)]
    fn alert(&mut self, when: Duration) -> io::Result<()> {
        self.set_nonblocking(false)?;
        self.set_read_timeout(Some(when))
    }
    #[inline(always)]
    fn sleep(&mut self) -> io::Result<()> {
        self.set_nonblocking(true)?;
        self.set_read_timeout(None)
    }
}

impl Close for TcpStream {
    fn close(&mut self) -> io::Result<()> {
        self.shutdown(Shutdown::Both)
    }
}

pub trait Config: Send + Sync + Clone + 'static {
    fn port(&self) -> u16;

    fn host(&self) -> &str;

    fn initial_coins_count(&self) -> u32;
}
