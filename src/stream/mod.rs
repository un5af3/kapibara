//! Stream for use

use tokio::io::{AsyncRead, AsyncWrite};

use std::time::Duration;

pub mod timer;
pub use timer::StreamTimer;

pub trait StreamTrait: AsyncRead + AsyncWrite + Send + Sync + Unpin {}
impl<S: AsyncRead + AsyncWrite + Send + Sync + Unpin> StreamTrait for S {}

pub trait ToStreamTimer: StreamTrait
where
    Self: Sized,
{
    fn to_timer(self, timeout: Option<Duration>) -> StreamTimer<Self> {
        StreamTimer::new(self, timeout)
    }
}
impl<S: StreamTrait> ToStreamTimer for S {}
