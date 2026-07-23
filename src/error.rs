use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("empty packet")]
    EmptyPacket,

    #[error("unknown packet type: {0}")]
    UnknownPacketType(u8),

    #[error("unknown error code: {0}")]
    UnknownErrorCode(i32),

    #[error("packet is {actual} bytes; maximum is {maximum}")]
    PacketTooLarge { actual: usize, maximum: usize },

    #[error("{field} is {actual} bytes; maximum is {maximum}")]
    FieldTooLarge {
        field: &'static str,
        actual: usize,
        maximum: usize,
    },

    #[error("room count {actual} exceeds maximum {maximum}")]
    TooManyRooms { actual: usize, maximum: usize },

    #[error("invalid boolean value {0}; expected 0 or 1")]
    InvalidBoolean(i32),

    #[error("packet has {0} trailing bytes")]
    TrailingBytes(usize),

    #[error("not enough bytes {0}")]
    NotEnoughBytes(String),

    #[error("failed to parse i32: {0}")]
    InvalidI32(#[from] std::array::TryFromSliceError),

    #[error("failed to parse UTF8 string: {0}")]
    InvalidUtf8String(#[from] std::string::FromUtf8Error),

    #[error("negative vector length")]
    NegativeVectorLength,
}
