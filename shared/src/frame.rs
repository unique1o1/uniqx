//! Shared data structures, utilities, and protocol definitions.

use anyhow::{Context, Error, Result};
use bytes::{Buf, BufMut, BytesMut};
use futures_util::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::timeout;
use tokio_util::codec::{BytesCodec, Framed};
use tracing::trace;

use crate::NETWORK_TIMEOUT;
/// Transport stream with JSON frames delimited by null characters.
pub struct Delimited<U>(Framed<U, BytesCodec>);

impl<U: AsyncRead + AsyncWrite + Unpin> Delimited<U> {
    /// Construct a new delimited stream.
    pub fn new(stream: U) -> Self {
        let codec = BytesCodec::default();
        Self(Framed::new(stream, codec))
    }

    /// Read the next null-delimited JSON instruction from a stream.
    pub async fn recv<T: DeserializeOwned>(&mut self) -> Result<T> {
        if let Some(next_message) = self.0.next().await {
            let byte_message = next_message.context("frame error, invalid byte length")?;

            match bincode::deserialize_from(byte_message.reader()) {
                Ok(msg) => Ok(msg),
                Err(e) => {
                    trace!("error deserializing message: {}", e);
                    Err(e.into())
                }
            }
        } else {
            Err(Error::msg("no message received"))
        }
    }

    /// Read the next null-delimited JSON instruction, with a default timeout.
    ///
    /// This is useful for parsing the initial message of a stream for handshake or
    /// other protocol purposes, where we do not want to wait indefinitely.
    pub async fn recv_timeout<T: DeserializeOwned>(&mut self) -> Result<T> {
        timeout(NETWORK_TIMEOUT, self.recv())
            .await
            .context("timed out waiting for initial message")?
    }

    /// Send a null-terminated JSON instruction on a stream.
    pub async fn send<T: Serialize>(&mut self, msg: T) -> Result<()> {
        trace!("sending json message");
        let mut writer = BytesMut::new().writer();
        bincode::serialize_into(&mut writer, &msg)?;
        self.0.send(writer.into_inner()).await?;
        Ok(())
    }

    // Consume this object, returning current buffers and the inner transport.
    // pub fn into_parts(self) -> FramedParts<U, AnyDelimiterCodec> {
    //     self.0.into_parts()
    // }
}
