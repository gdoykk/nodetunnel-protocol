use crate::error::ProtocolError;

/// Stable error values shared by the relay and Godot client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ErrorCode {
    InvalidRequest = 400,
    Unauthenticated = 401,
    Forbidden = 403,
    NotFound = 404,
    RequestTimeout = 408,
    Conflict = 409,
    RateLimited = 429,
    Internal = 500,
    Transport = 1000,
    Timeout = 1001,
    Protocol = 1002,
}

impl ErrorCode {
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

impl TryFrom<i32> for ErrorCode {
    type Error = ProtocolError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            400 => Self::InvalidRequest,
            401 => Self::Unauthenticated,
            403 => Self::Forbidden,
            404 => Self::NotFound,
            408 => Self::RequestTimeout,
            409 => Self::Conflict,
            429 => Self::RateLimited,
            500 => Self::Internal,
            1000 => Self::Transport,
            1001 => Self::Timeout,
            1002 => Self::Protocol,
            other => return Err(ProtocolError::UnknownErrorCode(other)),
        })
    }
}

impl From<ErrorCode> for i32 {
    fn from(value: ErrorCode) -> Self {
        value.as_i32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_error_code_round_trips() {
        let codes = [
            ErrorCode::InvalidRequest,
            ErrorCode::Unauthenticated,
            ErrorCode::Forbidden,
            ErrorCode::NotFound,
            ErrorCode::RequestTimeout,
            ErrorCode::Conflict,
            ErrorCode::RateLimited,
            ErrorCode::Internal,
            ErrorCode::Transport,
            ErrorCode::Timeout,
            ErrorCode::Protocol,
        ];

        for code in codes {
            assert_eq!(
                ErrorCode::try_from(code.as_i32()).expect("known error code"),
                code
            );
        }
        assert!(matches!(
            ErrorCode::try_from(402),
            Err(ProtocolError::UnknownErrorCode(402))
        ));
    }
}
