use crate::error::ProtocolError;

/// Identifies a client's transport-level session with the relay.
///
/// This is distinct from the per-room Godot `MultiplayerPeer` id (an `i32`
/// assigned by the room when a client joins). `ClientId` is stable for the
/// lifetime of a client's UDP session with the relay; the Godot peer id is
/// scoped to a single room.
///
/// Wrapping this in a newtype (rather than passing a bare `u64` around)
/// prevents accidentally mixing it up with room ids, app ids, or Godot peer
/// ids in function signatures that take several integer parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClientId(pub u64);

impl ClientId {
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for ClientId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<ClientId> for u64 {
    fn from(value: ClientId) -> Self {
        value.0
    }
}

pub fn read_client_id(bytes: &[u8]) -> Result<(ClientId, &[u8]), ProtocolError> {
    let (raw, rest) = crate::serialize::read_u64(bytes)?;
    Ok((ClientId(raw), rest))
}

pub fn push_client_id(buf: &mut Vec<u8>, value: ClientId) {
    crate::serialize::push_u64(buf, value.0);
}
