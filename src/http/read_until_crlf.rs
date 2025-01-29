use tokio::io::AsyncBufRead;

use pin_project_lite::pin_project;
use std::future::Future;
use std::io;
use std::marker::PhantomPinned;
use std::mem;
use std::pin::Pin;
use std::task::{ready, Context, Poll};

fn find_crlf_in_bytes(bytes: &[u8]) -> Option<usize> {
    enum State {
        ExpectCR,
        ExpectLF,
    }
    let mut state = State::ExpectCR;
    for i in 0..bytes.len() {
        match state {
            State::ExpectCR => {
                if bytes[i] == b'\r' {
                    state = State::ExpectLF;
                }
            },
            State::ExpectLF => {
                if bytes[i] == b'\n' {
                    return Some(i-1);
                }
            }
        }
    };

    None
}

pin_project! {
    /// Future for the [`read_until_crlf`] method.
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct ReadUntilCrlf<'a, R: ?Sized> {
        reader: &'a mut R,
        buf: &'a mut Vec<u8>,
        // The number of bytes appended to buf. This can be less than buf.len() if
        // the buffer was not empty when the operation was started.
        read: usize,
        // Make this future `!Unpin` for compatibility with async trait methods.
        #[pin]
        _pin: PhantomPinned,
    }
}

pub(crate) fn read_until_crlf<'a, R>(
    reader: &'a mut R,
    buf: &'a mut Vec<u8>,
) -> ReadUntilCrlf<'a, R>
where
    R: AsyncBufRead + ?Sized + Unpin,
{
    ReadUntilCrlf {
        reader,
        buf,
        read: 0,
        _pin: PhantomPinned,
    }
}

pub(super) fn read_until_crlf_internal<R: AsyncBufRead + ?Sized>(
    mut reader: Pin<&mut R>,
    cx: &mut Context<'_>,
    buf: &mut Vec<u8>,
    read: &mut usize,
) -> Poll<io::Result<usize>> {
    loop {
        let (done, used) = {
            let available = ready!(reader.as_mut().poll_fill_buf(cx))?;
            if let Some(i) = find_crlf_in_bytes(available) {
                buf.extend_from_slice(&available[..=i + 1]);
                (true, i + 2)
            } else if buf.len() > 0
                && available.len() > 0
                && buf[0] == b'\r'
                && available[0] == b'\n'
            {
                buf.push(available[0]);
                (true, 1)
            } else {
                buf.extend_from_slice(available);
                (false, available.len())
            }
        };
        reader.as_mut().consume(used);
        *read += used;
        if done || used == 0 {
            return Poll::Ready(Ok(mem::replace(read, 0)));
        }
    }
}

impl<R: AsyncBufRead + ?Sized + Unpin> Future for ReadUntilCrlf<'_, R> {
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        read_until_crlf_internal(Pin::new(*me.reader), cx, me.buf, me.read)
    }
}


pub trait AsyncBufReadUtilCrlf: AsyncBufRead {
    fn read_until_crlf<'a>(&'a mut self, buf: &'a mut Vec<u8>) -> ReadUntilCrlf<'a, Self>
    where
        Self: Unpin,
    {
        read_until_crlf(self, buf)
    }

}

impl<R: AsyncBufRead + ?Sized> AsyncBufReadUtilCrlf for R {}
