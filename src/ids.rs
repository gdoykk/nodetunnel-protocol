use crate::error::ProtocolError;

/// Identifies the kind of a [`crate::packet::Packet`] on the wire.
///
/// This is a real enum (rather than a set of loose `u8` constants) so that:
/// - the compiler enforces exhaustive handling wherever a `PacketKind` is
///   matched on,
/// - invalid bytes are rejected by a single `TryFrom` conversion instead of
///   a hand-written `_ => Err(...)` fallback that has to be kept in sync,
/// - the discriminant <-> variant mapping lives in exactly one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PacketKind {
    Authenticate = 0,
    ClientAuthenticated = 1,
    CreateRoom = 2,
    JoinRoom = 3,
    ConnectedToRoom = 4,
    PeerJoined = 5,
    PeerLeft = 6,
    GameData = 7,
    ForceDisconnect = 8,
    ErrorPacket = 9,
    ReqRooms = 10,
    GetRooms = 11,
    UpdateRoom = 12,
    JoinRes = 13,
    PeerJoinAttempt = 14,
}

impl PacketKind {
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for PacketKind {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Authenticate,
            1 => Self::ClientAuthenticated,
            2 => Self::CreateRoom,
            3 => Self::JoinRoom,
            4 => Self::ConnectedToRoom,
            5 => Self::PeerJoined,
            6 => Self::PeerLeft,
            7 => Self::GameData,
            8 => Self::ForceDisconnect,
            9 => Self::ErrorPacket,
            10 => Self::ReqRooms,
            11 => Self::GetRooms,
            12 => Self::UpdateRoom,
            13 => Self::JoinRes,
            14 => Self::PeerJoinAttempt,
            other => return Err(ProtocolError::UnknownPacketType(other)),
        })
    }
}

impl From<PacketKind> for u8 {
    fn from(value: PacketKind) -> Self {
        value.as_u8()
    }
}
