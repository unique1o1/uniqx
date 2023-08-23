use crate::NETWORK_TIMEOUT;
use anyhow::{Context, Error, Result};
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std;
use tokio::io::{AsyncRead, AsyncWrite, WriteHalf};
use tokio::time::timeout;
use tokio::{io::ReadHalf, net::TcpStream};
use tokio_util::codec::{
    AnyDelimiterCodec, AnyDelimiterCodecError, Framed, FramedRead, FramedWrite,
};
use tracing::trace;

pub type DelimitedStream<T = TcpStream> = Framed<T, AnyDelimiterCodec>;
pub type DelimitedWriteStream<T = WriteHalf<TcpStream>> = FramedWrite<T, AnyDelimiterCodec>;
pub type DelimitedReadStream<T = ReadHalf<TcpStream>> = FramedRead<T, AnyDelimiterCodec>;

pub fn delimited_framed<T: AsyncWrite + AsyncRead>(stream: T) -> DelimitedStream<T> {
    let codec = AnyDelimiterCodec::new_with_max_length(vec![0], vec![0], 20000);
    DelimitedStream::new(stream, codec)
}
pub fn delimited_framed_write<T: AsyncWrite>(stream: T) -> DelimitedWriteStream<T> {
    let codec = AnyDelimiterCodec::new_with_max_length(vec![0], vec![0], 20000);
    DelimitedWriteStream::new(stream, codec)
}
pub fn delimited_framed_read<T: AsyncRead>(stream: T) -> DelimitedReadStream<T> {
    let codec = AnyDelimiterCodec::new_with_max_length(vec![0], vec![0], 20000);
    DelimitedReadStream::new(stream, codec)
}
#[async_trait]
pub trait DelimitedWriteExt<T> {
    async fn send_delimited(&mut self, msg: T) -> Result<()>;
}
#[async_trait]
impl<U, T> DelimitedWriteExt<T> for U
where
    T: Serialize + Send + Sync + 'static,
    U: Sink<String> + SinkExt<String> + Unpin + Send,
    <U as Sink<String>>::Error: std::error::Error + Send + Sync + 'static,
{
    async fn send_delimited(&mut self, msg: T) -> Result<()> {
        trace!("sending json message");
        self.send(serde_json::to_string(&msg)?).await?;
        Ok(())
    }
}

#[async_trait]
pub trait DelimitedReadExt {
    async fn recv_delimited<S: DeserializeOwned + Send + Sync + 'static>(&mut self) -> Result<S>;
    async fn recv_timeout_delimited<S: DeserializeOwned + Send + Sync + 'static>(
        &mut self,
    ) -> Result<S>;
}
#[async_trait]
impl<U> DelimitedReadExt for U
where
    U: Stream + StreamExt + Unpin + Send,
    U: Stream<Item = Result<Bytes, AnyDelimiterCodecError>>,
{
    /// Read the next null-delimited JSON instruction, with a default timeout.
    ///
    /// This is useful for parsing the initial message of a stream for handshake or
    /// other protocol purposes, where we do not want to wait indefinitely.
    async fn recv_timeout_delimited<S: DeserializeOwned + Send + Sync + 'static>(
        &mut self,
    ) -> Result<S> {
        timeout(NETWORK_TIMEOUT, self.recv_delimited())
            .await
            .context("timed out waiting for initial message")?
    }

    async fn recv_delimited<S: DeserializeOwned + Send + Sync + 'static>(&mut self) -> Result<S> {
        if let Some(next_message) = self.next().await {
            let byte_message = next_message.context("frame error, invalid byte length")?;
            let serialized_obj =
                serde_json::from_slice(&byte_message).context("unable to parse message")?;
            Ok(serialized_obj)
        } else {
            Err(Error::msg("Connection closed"))
        }
    }
}
