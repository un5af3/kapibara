//! Copy Stream

use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
};

const DEFAULT_BUF_SIZE: usize = 8 * 1024;

use futures_util::{future::poll_fn, ready};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub async fn copy_bi<A, B>(a: &mut A, b: &mut B) -> io::Result<(u64, u64)>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    poll_fn(|cx| {
        let a_to_b = copy(a, b).poll_copy(cx);
        let b_to_a = copy(b, a).poll_copy(cx);

        let a_to_b = ready!(a_to_b)?;
        let b_to_a = ready!(b_to_a)?;

        Poll::Ready(Ok((a_to_b, b_to_a)))
    })
    .await
}

pub async fn copy_bi_with_size<A, B>(
    a: &mut A,
    b: &mut B,
    a_to_b_buf_size: usize,
    b_to_a_buf_size: usize,
) -> io::Result<(u64, u64)>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    poll_fn(|cx| {
        let a_to_b = copy_with_size(a, b, a_to_b_buf_size).poll_copy(cx);
        let b_to_a = copy_with_size(b, a, b_to_a_buf_size).poll_copy(cx);

        let a_to_b = ready!(a_to_b)?;
        let b_to_a = ready!(b_to_a)?;

        Poll::Ready(Ok((a_to_b, b_to_a)))
    })
    .await
}

pub fn copy<'a, R, W>(reader: &'a mut R, writer: &'a mut W) -> Copy<'a, R, W>
where
    R: AsyncRead + Unpin + ?Sized,
    W: AsyncWrite + Unpin + ?Sized,
{
    copy_with_size(reader, writer, DEFAULT_BUF_SIZE)
}

pub fn copy_with_size<'a, R, W>(
    reader: &'a mut R,
    writer: &'a mut W,
    buf_size: usize,
) -> Copy<'a, R, W>
where
    R: AsyncRead + Unpin + ?Sized,
    W: AsyncWrite + Unpin + ?Sized,
{
    Copy::new(buf_size, Pin::new(reader), Pin::new(writer))
}

pub struct Copy<'a, R, W>
where
    R: AsyncRead + ?Sized,
    W: AsyncWrite + ?Sized,
{
    reader: Pin<&'a mut R>,
    writer: Pin<&'a mut W>,
    amt: u64,
    cap: usize,
    pos: usize,
    buf: Box<[u8]>,
    state: CopyState,
}

enum CopyState {
    Read,
    Write,
    Flush,
    Done,
}

impl<'a, R, W> Copy<'a, R, W>
where
    R: AsyncRead + ?Sized,
    W: AsyncWrite + ?Sized,
{
    pub fn new(buf_size: usize, reader: Pin<&'a mut R>, writer: Pin<&'a mut W>) -> Self {
        Self {
            reader,
            writer,
            amt: 0,
            cap: 0,
            pos: 0,
            buf: vec![0; buf_size].into_boxed_slice(),
            state: CopyState::Read,
        }
    }

    fn poll_read_buf(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let mut buf = ReadBuf::new(&mut self.buf);
        buf.set_filled(self.cap);

        let res = self.reader.as_mut().poll_read(cx, &mut buf);
        if let Poll::Ready(Ok(())) = res {
            let filled_len = buf.filled().len();
            if self.cap == filled_len {
                self.state = CopyState::Done;
            } else {
                self.state = CopyState::Write;
            }
            self.cap = filled_len;
        }

        res
    }

    fn poll_write_buf(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        while self.pos < self.cap {
            let n = ready!(self
                .writer
                .as_mut()
                .poll_write(cx, &self.buf[self.pos..self.cap]))?;

            if n == 0 {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "write zero byte into writer",
                )));
            } else {
                self.pos += n;
                self.amt += n as u64;
            }
        }

        self.pos = 0;
        self.cap = 0;
        self.state = CopyState::Flush;

        Poll::Ready(Ok(()))
    }

    fn poll_flush_buf(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        ready!(self.writer.as_mut().poll_flush(cx))?;

        self.state = CopyState::Read;

        Poll::Ready(Ok(()))
    }

    fn poll_copy(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
        loop {
            match self.state {
                CopyState::Read => ready!(self.poll_read_buf(cx))?,
                CopyState::Done => return Poll::Ready(Ok(self.amt)),
                CopyState::Write => ready!(self.poll_write_buf(cx))?,
                CopyState::Flush => ready!(self.poll_flush_buf(cx))?,
            }
        }
    }
}

impl<'a, R, W> Future for Copy<'a, R, W>
where
    R: AsyncRead + ?Sized,
    W: AsyncWrite + ?Sized,
{
    type Output = io::Result<u64>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().poll_copy(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::io::{duplex, AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_copy() {
        let (mut in_tx, mut in_rx) = duplex(DEFAULT_BUF_SIZE);
        let (mut out_tx, mut out_rx) = duplex(DEFAULT_BUF_SIZE);

        let h1 = tokio::spawn(async move {
            let n = copy(&mut in_rx, &mut out_tx).await.unwrap();
            assert_eq!(n, 4 * 1024 * 100)
        });

        let h2 = tokio::spawn(async move {
            for _ in 0..100 {
                in_tx.write(&b"test".repeat(1024)).await.unwrap();
            }
        });

        let mut buf = [0u8; 4 * 1024];
        for _ in 0..100 {
            match out_rx.read(&mut buf).await {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                }
                Err(e) => println!("error: {}", e),
            }
        }

        let _ = h2.await;
        let _ = h1.await;
    }
}
