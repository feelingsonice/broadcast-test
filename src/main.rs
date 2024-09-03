use bytes::{BufMut, Bytes, BytesMut};
use fred::prelude::*;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct TileId {
    pub z: u32,
    pub x: u32,
    pub y: u32,
}

fn bytes_to_u32(bytes: &[u8]) -> Result<u32, std::io::Error> {
    bytes
        .try_into()
        .map(u32::from_be_bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

impl TryFrom<Bytes> for TileId {
    type Error = std::io::Error;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.len() != 12 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid byte length to convert to TileId",
            ));
        }
        Ok(TileId {
            z: bytes_to_u32(&value[0..4])?,
            x: bytes_to_u32(&value[4..8])?,
            y: bytes_to_u32(&value[8..12])?,
        })
    }
}

impl From<TileId> for Bytes {
    fn from(value: TileId) -> Self {
        let mut bytes = BytesMut::with_capacity(12);
        bytes.put_u32(value.z);
        bytes.put_u32(value.x);
        bytes.put_u32(value.y);
        bytes.freeze()
    }
}

impl FromRedis for TileId {
    fn from_value(value: RedisValue) -> Result<Self, RedisError> {
        println!("Calling value, got: {:?}", value);
        match value {
            RedisValue::Bytes(b) => b.try_into().map_err(|e| {
                RedisError::new(
                    RedisErrorKind::Parse,
                    format!("Failed parsing TileId from bytes: {}", e),
                )
            }),
            _ => Err(RedisError::new(
                RedisErrorKind::Parse,
                "Expected RedisValue::Bytes type",
            )),
        }
    }
}

// CONVERTION TO REDIS VALUE
impl From<TileId> for RedisValue {
    fn from(value: TileId) -> Self {
        let bytes: Bytes = value.into();
        bytes.into()
    }
}

impl From<TileId> for RedisKey {
    fn from(value: TileId) -> Self {
        let bytes: Bytes = value.into();
        bytes.into()
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let publisher_client = RedisClient::default();
    let subscriber_client = publisher_client.clone_new();
    publisher_client.init().await?;
    subscriber_client.init().await?;

    subscriber_client.subscribe("foo").await?;
    let _message_task = subscriber_client.on_message(|message| {
        println!(
            "{}: {:?}",
            message.channel,
            message.value.convert::<TileId>()? // !!!!! SHOULD ERRROR HERE !!!!!
        );
        Ok::<_, RedisError>(())
    });

    for idx in 0..50 {
        let id = TileId { z: idx, x: 2, y: 3 }; // ***** SENDING A BYTE TYPE *****
        publisher_client.publish("foo", id).await?;
    }

    // SHOULD PRINT ERROR BUT DID NOT
    let _r = _message_task.await.expect("BIG ERROR");

    publisher_client.quit().await?;
    subscriber_client.quit().await?;
    Ok(())
}
