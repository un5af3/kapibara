use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
    time::Duration,
};

use pin_project_lite::pin_project;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    time,
};

use super::StreamTrait;

pin_project! {
    pub struct StreamTimer<S: StreamTrait> {
        #[pin]
        inner: S,
        timeout: Option<Duration>,
        #[pin]
        timer: Option<Pin<Box<time::Sleep>>>,
    }
}

impl<S: StreamTrait> StreamTimer<S> {
    pub fn new(inner: S, timeout: Option<Duration>) -> Self {
        Self {
            inner,
            timeout,
            timer: None,
        }
    }

    pub fn inner(self) -> S {
        self.inner
    }

    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.timeout = timeout;
    }
}

impl<S: StreamTrait> AsyncRead for StreamTimer<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let mut this = self.project();

        if let Some(timeout) = this.timeout {
            if let Poll::Ready(res) = this.inner.poll_read(cx, buf) {
                return Poll::Ready(res);
            }

            if this.timer.is_none() {
                this.timer.set(Some(Box::pin(time::sleep(*timeout))));
            }

            if let Some(sleep) = this.timer.as_mut().as_pin_mut() {
                if let Poll::Ready(_) = sleep.poll(cx) {
                    this.timer.set(None);
                    return Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "timedout",
                    )));
                }
            }

            Poll::Pending
        } else {
            this.inner.poll_read(cx, buf)
        }
    }
}

impl<S: StreamTrait> AsyncWrite for StreamTimer<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        self.project().inner.poll_write(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.project().inner.poll_shutdown(cx)
    }
}
