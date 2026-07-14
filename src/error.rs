use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("empty packet")]
    EmptyPacket,

    #[error("unknown packet type: {0}")]
    UnknownPacketType(u8),

    #[error("not enough bytes {0}")]
    NotEnoughBytes(String),

    #[error("failed to parse i32: {0}")]
    InvalidI32(#[from] std::array::TryFromSliceError),

    #[error("failed to parse UTF8 string: {0}")]
    InvalidUtf8String(#[from] std::string::FromUtf8Error),

    #[error("negative vector length")]
    NegativeVectorLength,
}
