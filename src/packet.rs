use crate::client_id::{push_client_id, read_client_id, ClientId};
use crate::error::ProtocolError;
use crate::ids::PacketKind;
use crate::serialize::{
    push_bool, push_i32, push_string, push_vec_room_info, read_bool, read_i32, read_string,
    read_vec_room_info,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomInfo {
    pub join_code: String,
    pub metadata: String,
}

#[derive(Debug, Clone)]
pub enum Packet {
    Authenticate { app_id: String, version: String },
    ClientAuthenticated,
    CreateRoom { is_public: bool, metadata: String },
    ReqRooms,
    GetRooms { rooms: Vec<RoomInfo> },
    UpdateRoom { room_id: String, metadata: String },
    ReqJoin { room_id: String, metadata: String },
    JoinRes { target_id: ClientId, room_id: String, allowed: bool },
    ConnectedToRoom { room_id: String, peer_id: i32 },
    PeerJoinAttempt { target_id: ClientId, metadata: String },
    PeerJoinedRoom { peer_id: i32 },
    PeerLeftRoom { peer_id: i32 },
    GameData { from_peer: i32, data: Vec<u8> },
    ForceDisconnect,
    Error { error_code: i32, error_message: String },
}

impl Packet {
    /// Returns the wire [`PacketKind`] of this packet without needing to
    /// destructure it, e.g. for logging.
    #[must_use]
    pub const fn kind(&self) -> PacketKind {
        match self {
            Packet::Authenticate { .. } => PacketKind::Authenticate,
            Packet::ClientAuthenticated => PacketKind::ClientAuthenticated,
            Packet::CreateRoom { .. } => PacketKind::CreateRoom,
            Packet::ReqRooms => PacketKind::ReqRooms,
            Packet::GetRooms { .. } => PacketKind::GetRooms,
            Packet::UpdateRoom { .. } => PacketKind::UpdateRoom,
            Packet::ReqJoin { .. } => PacketKind::JoinRoom,
            Packet::JoinRes { .. } => PacketKind::JoinRes,
            Packet::ConnectedToRoom { .. } => PacketKind::ConnectedToRoom,
            Packet::PeerJoinAttempt { .. } => PacketKind::PeerJoinAttempt,
            Packet::PeerJoinedRoom { .. } => PacketKind::PeerJoined,
            Packet::PeerLeftRoom { .. } => PacketKind::PeerLeft,
            Packet::GameData { .. } => PacketKind::GameData,
            Packet::ForceDisconnect => PacketKind::ForceDisconnect,
            Packet::Error { .. } => PacketKind::ErrorPacket,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let Some((&packet_id, rest)) = bytes.split_first() else {
            return Err(ProtocolError::EmptyPacket);
        };

        let kind = PacketKind::try_from(packet_id)?;

        Ok(match kind {
            PacketKind::Authenticate => {
                let (app_id, r) = read_string(rest)?;
                let (version, _) = read_string(r)?;
                Packet::Authenticate { app_id, version }
            }

            PacketKind::ClientAuthenticated => Packet::ClientAuthenticated,

            PacketKind::CreateRoom => {
                let (is_public, r) = read_bool(rest)?;
                let metadata = read_string(r).map(|(name, _)| name).unwrap_or_default();

                Packet::CreateRoom { is_public, metadata }
            }

            PacketKind::JoinRoom => {
                let (room_id, r) = read_string(rest)?;
                let (metadata, _) = read_string(r)?;
                Packet::ReqJoin { room_id, metadata }
            }

            PacketKind::ConnectedToRoom => {
                let (room_id, r) = read_string(rest)?;
                let (peer_id, _) = read_i32(r)?;
                Packet::ConnectedToRoom { room_id, peer_id }
            }

            PacketKind::PeerJoinAttempt => {
                let (target_id, r) = read_client_id(rest)?;
                let (metadata, _) = read_string(r)?;
                Packet::PeerJoinAttempt { target_id, metadata }
            }

            PacketKind::PeerJoined => {
                let (peer_id, _) = read_i32(rest)?;
                Packet::PeerJoinedRoom { peer_id }
            }

            PacketKind::PeerLeft => {
                let (peer_id, _) = read_i32(rest)?;
                Packet::PeerLeftRoom { peer_id }
            }

            PacketKind::GameData => {
                let (peer_id, r) = read_i32(rest)?;
                Packet::GameData { from_peer: peer_id, data: r.to_vec() }
            }

            PacketKind::ForceDisconnect => Packet::ForceDisconnect,

            PacketKind::ErrorPacket => {
                let (error_code, r) = read_i32(rest)?;
                let (error_message, _) = read_string(r)?;
                Packet::Error { error_code, error_message }
            }

            PacketKind::ReqRooms => Packet::ReqRooms,

            PacketKind::GetRooms => {
                let (rooms, _) = read_vec_room_info(rest)?;
                Packet::GetRooms { rooms }
            }

            PacketKind::UpdateRoom => {
                let (room_id, r) = read_string(rest)?;
                let (metadata, _) = read_string(r)?;
                Packet::UpdateRoom { room_id, metadata }
            }

            PacketKind::JoinRes => {
                let (target_id, r) = read_client_id(rest)?;
                let (room_id, r) = read_string(r)?;
                let (allowed, _) = read_bool(r)?;
                Packet::JoinRes { target_id, room_id, allowed }
            }
        })
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.kind().as_u8());

        match self {
            Packet::Authenticate { app_id, version } => {
                push_string(&mut buf, app_id);
                push_string(&mut buf, version);
            }

            Packet::ClientAuthenticated => {}

            Packet::CreateRoom { is_public, metadata } => {
                push_bool(&mut buf, *is_public);
                push_string(&mut buf, metadata);
            }

            Packet::ReqRooms => {}

            Packet::GetRooms { rooms } => {
                push_vec_room_info(&mut buf, rooms);
            }

            Packet::UpdateRoom { room_id, metadata } => {
                push_string(&mut buf, room_id);
                push_string(&mut buf, metadata);
            }

            Packet::ReqJoin { room_id, metadata } => {
                push_string(&mut buf, room_id);
                push_string(&mut buf, metadata);
            }

            Packet::JoinRes { target_id, room_id, allowed } => {
                push_client_id(&mut buf, *target_id);
                push_string(&mut buf, room_id);
                push_bool(&mut buf, *allowed);
            }

            Packet::ConnectedToRoom { room_id, peer_id } => {
                push_string(&mut buf, room_id);
                push_i32(&mut buf, *peer_id);
            }

            Packet::PeerJoinAttempt { target_id, metadata } => {
                push_client_id(&mut buf, *target_id);
                push_string(&mut buf, metadata);
            }

            Packet::PeerJoinedRoom { peer_id } => {
                push_i32(&mut buf, *peer_id);
            }

            Packet::PeerLeftRoom { peer_id } => {
                push_i32(&mut buf, *peer_id);
            }

            Packet::GameData { from_peer: peer_id, data } => {
                push_i32(&mut buf, *peer_id);
                buf.extend(data);
            }

            Packet::ForceDisconnect => {}

            Packet::Error { error_code, error_message } => {
                push_i32(&mut buf, *error_code);
                push_string(&mut buf, error_message);
            }
        }

        buf
    }
}
