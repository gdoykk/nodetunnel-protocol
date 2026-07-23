use crate::client_id::{ClientId, push_client_id, read_client_id};
use crate::error::ProtocolError;
use crate::error_code::ErrorCode;
use crate::ids::PacketKind;
use crate::serialize::{
    push_bool, push_fixed_32, push_i32, push_string, push_u64, push_vec_room_info, read_bool,
    read_fixed_32, read_i32, read_string, read_u64, read_vec_room_info,
};

pub const MAX_PROTOCOL_PACKET_BYTES: usize = 1_187;
pub const MAX_APP_ID_BYTES: usize = 64;
pub const MAX_VERSION_BYTES: usize = 32;
pub const MAX_ROOM_CODE_BYTES: usize = 26;
pub const MAX_ROOM_METADATA_BYTES: usize = 256;
pub const MAX_JOIN_METADATA_BYTES: usize = 256;
pub const MAX_ERROR_MESSAGE_BYTES: usize = 256;
pub const MAX_GAME_DATA_BYTES: usize = 1_182;
pub const MAX_ROOMS_PER_PAGE: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomInfo {
    pub join_code: String,
    pub metadata: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Packet {
    Authenticate {
        nonce: u64,
        seed: [u8; 32],
        cookie: [u8; 32],
        app_id: String,
        version: String,
    },
    ClientAuthenticated,
    CreateRoom {
        is_public: bool,
        max_peers: i32,
        metadata: String,
    },
    ReqRooms {
        cursor: u64,
        snapshot_end: u64,
    },
    GetRooms {
        rooms: Vec<RoomInfo>,
        next_cursor: u64,
        snapshot_end: u64,
    },
    UpdateRoom {
        room_id: String,
        metadata: String,
    },
    ReqJoin {
        room_id: String,
        metadata: String,
    },
    JoinRes {
        target_id: ClientId,
        room_id: String,
        allowed: bool,
    },
    ConnectedToRoom {
        room_id: String,
        peer_id: i32,
    },
    PeerJoinAttempt {
        target_id: ClientId,
        metadata: String,
    },
    PeerJoinedRoom {
        peer_id: i32,
    },
    PeerLeftRoom {
        peer_id: i32,
    },
    GameData {
        from_peer: i32,
        data: Vec<u8>,
    },
    ForceDisconnect,
    Error {
        error_code: ErrorCode,
        error_message: String,
    },
    Hello {
        nonce: u64,
        seed: [u8; 32],
    },
    Cookie {
        nonce: u64,
        token: [u8; 32],
    },
    Ping {
        nonce: u64,
    },
    Pong {
        nonce: u64,
    },
    Disconnect,
    DisconnectPeer {
        peer_id: i32,
    },
    SendFailed {
        target_peer: i32,
        reason: ErrorCode,
    },
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
            Packet::ReqRooms { .. } => PacketKind::ReqRooms,
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
            Packet::Hello { .. } => PacketKind::Hello,
            Packet::Cookie { .. } => PacketKind::Cookie,
            Packet::Ping { .. } => PacketKind::Ping,
            Packet::Pong { .. } => PacketKind::Pong,
            Packet::Disconnect => PacketKind::Disconnect,
            Packet::DisconnectPeer { .. } => PacketKind::DisconnectPeer,
            Packet::SendFailed { .. } => PacketKind::SendFailed,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() > MAX_PROTOCOL_PACKET_BYTES {
            return Err(ProtocolError::PacketTooLarge {
                actual: bytes.len(),
                maximum: MAX_PROTOCOL_PACKET_BYTES,
            });
        }

        let Some((&packet_id, rest)) = bytes.split_first() else {
            return Err(ProtocolError::EmptyPacket);
        };

        let kind = PacketKind::try_from(packet_id)?;
        let (packet, remaining) = match kind {
            PacketKind::Authenticate => {
                let (nonce, r) = read_u64(rest)?;
                let (seed, r) = read_fixed_32(r)?;
                let (cookie, r) = read_fixed_32(r)?;
                let (app_id, r) = read_string(r, MAX_APP_ID_BYTES, "app id")?;
                let (version, r) = read_string(r, MAX_VERSION_BYTES, "version")?;
                (
                    Packet::Authenticate {
                        nonce,
                        seed,
                        cookie,
                        app_id,
                        version,
                    },
                    r,
                )
            }
            PacketKind::ClientAuthenticated => (Packet::ClientAuthenticated, rest),
            PacketKind::CreateRoom => {
                let (is_public, r) = read_bool(rest)?;
                let (max_peers, r) = read_i32(r)?;
                let (metadata, r) = read_string(r, MAX_ROOM_METADATA_BYTES, "room metadata")?;
                (
                    Packet::CreateRoom {
                        is_public,
                        max_peers,
                        metadata,
                    },
                    r,
                )
            }
            PacketKind::JoinRoom => {
                let (room_id, r) = read_string(rest, MAX_ROOM_CODE_BYTES, "room code")?;
                let (metadata, r) = read_string(r, MAX_JOIN_METADATA_BYTES, "join metadata")?;
                (Packet::ReqJoin { room_id, metadata }, r)
            }
            PacketKind::ConnectedToRoom => {
                let (room_id, r) = read_string(rest, MAX_ROOM_CODE_BYTES, "room code")?;
                let (peer_id, r) = read_i32(r)?;
                (Packet::ConnectedToRoom { room_id, peer_id }, r)
            }
            PacketKind::PeerJoinAttempt => {
                let (target_id, r) = read_client_id(rest)?;
                let (metadata, r) = read_string(r, MAX_JOIN_METADATA_BYTES, "join metadata")?;
                (
                    Packet::PeerJoinAttempt {
                        target_id,
                        metadata,
                    },
                    r,
                )
            }
            PacketKind::PeerJoined => {
                let (peer_id, r) = read_i32(rest)?;
                (Packet::PeerJoinedRoom { peer_id }, r)
            }
            PacketKind::PeerLeft => {
                let (peer_id, r) = read_i32(rest)?;
                (Packet::PeerLeftRoom { peer_id }, r)
            }
            PacketKind::GameData => {
                let (peer_id, data) = read_i32(rest)?;
                if data.len() > MAX_GAME_DATA_BYTES {
                    return Err(ProtocolError::FieldTooLarge {
                        field: "game data",
                        actual: data.len(),
                        maximum: MAX_GAME_DATA_BYTES,
                    });
                }
                (
                    Packet::GameData {
                        from_peer: peer_id,
                        data: data.to_vec(),
                    },
                    &[][..],
                )
            }
            PacketKind::ForceDisconnect => (Packet::ForceDisconnect, rest),
            PacketKind::ErrorPacket => {
                let (error_code, r) = read_i32(rest)?;
                let error_code = ErrorCode::try_from(error_code)?;
                let (error_message, r) = read_string(r, MAX_ERROR_MESSAGE_BYTES, "error message")?;
                (
                    Packet::Error {
                        error_code,
                        error_message,
                    },
                    r,
                )
            }
            PacketKind::ReqRooms => {
                let (cursor, r) = read_u64(rest)?;
                let (snapshot_end, r) = read_u64(r)?;
                (
                    Packet::ReqRooms {
                        cursor,
                        snapshot_end,
                    },
                    r,
                )
            }
            PacketKind::GetRooms => {
                let (rooms, r) = read_vec_room_info(rest)?;
                let (next_cursor, r) = read_u64(r)?;
                let (snapshot_end, r) = read_u64(r)?;
                (
                    Packet::GetRooms {
                        rooms,
                        next_cursor,
                        snapshot_end,
                    },
                    r,
                )
            }
            PacketKind::UpdateRoom => {
                let (room_id, r) = read_string(rest, MAX_ROOM_CODE_BYTES, "room code")?;
                let (metadata, r) = read_string(r, MAX_ROOM_METADATA_BYTES, "room metadata")?;
                (Packet::UpdateRoom { room_id, metadata }, r)
            }
            PacketKind::JoinRes => {
                let (target_id, r) = read_client_id(rest)?;
                let (room_id, r) = read_string(r, MAX_ROOM_CODE_BYTES, "room code")?;
                let (allowed, r) = read_bool(r)?;
                (
                    Packet::JoinRes {
                        target_id,
                        room_id,
                        allowed,
                    },
                    r,
                )
            }
            PacketKind::Hello => {
                let (nonce, r) = read_u64(rest)?;
                let (seed, r) = read_fixed_32(r)?;
                (Packet::Hello { nonce, seed }, r)
            }
            PacketKind::Cookie => {
                let (nonce, r) = read_u64(rest)?;
                let (token, r) = read_fixed_32(r)?;
                (Packet::Cookie { nonce, token }, r)
            }
            PacketKind::Ping => {
                let (nonce, r) = read_u64(rest)?;
                (Packet::Ping { nonce }, r)
            }
            PacketKind::Pong => {
                let (nonce, r) = read_u64(rest)?;
                (Packet::Pong { nonce }, r)
            }
            PacketKind::Disconnect => (Packet::Disconnect, rest),
            PacketKind::DisconnectPeer => {
                let (peer_id, r) = read_i32(rest)?;
                (Packet::DisconnectPeer { peer_id }, r)
            }
            PacketKind::SendFailed => {
                let (target_peer, r) = read_i32(rest)?;
                let (reason, r) = read_i32(r)?;
                let reason = ErrorCode::try_from(reason)?;
                (
                    Packet::SendFailed {
                        target_peer,
                        reason,
                    },
                    r,
                )
            }
        };

        if !remaining.is_empty() {
            return Err(ProtocolError::TrailingBytes(remaining.len()));
        }
        Ok(packet)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ProtocolError> {
        let encoded_len = self.encoded_len()?;
        let mut buf = Vec::with_capacity(encoded_len);
        buf.push(self.kind().as_u8());

        match self {
            Packet::Authenticate {
                nonce,
                seed,
                cookie,
                app_id,
                version,
            } => {
                push_u64(&mut buf, *nonce);
                push_fixed_32(&mut buf, seed);
                push_fixed_32(&mut buf, cookie);
                push_string(&mut buf, app_id);
                push_string(&mut buf, version);
            }
            Packet::ClientAuthenticated | Packet::ForceDisconnect | Packet::Disconnect => {}
            Packet::CreateRoom {
                is_public,
                max_peers,
                metadata,
            } => {
                push_bool(&mut buf, *is_public);
                push_i32(&mut buf, *max_peers);
                push_string(&mut buf, metadata);
            }
            Packet::ReqRooms {
                cursor,
                snapshot_end,
            } => {
                push_u64(&mut buf, *cursor);
                push_u64(&mut buf, *snapshot_end);
            }
            Packet::GetRooms {
                rooms,
                next_cursor,
                snapshot_end,
            } => {
                push_vec_room_info(&mut buf, rooms);
                push_u64(&mut buf, *next_cursor);
                push_u64(&mut buf, *snapshot_end);
            }
            Packet::UpdateRoom { room_id, metadata } | Packet::ReqJoin { room_id, metadata } => {
                push_string(&mut buf, room_id);
                push_string(&mut buf, metadata);
            }
            Packet::JoinRes {
                target_id,
                room_id,
                allowed,
            } => {
                push_client_id(&mut buf, *target_id);
                push_string(&mut buf, room_id);
                push_bool(&mut buf, *allowed);
            }
            Packet::ConnectedToRoom { room_id, peer_id } => {
                push_string(&mut buf, room_id);
                push_i32(&mut buf, *peer_id);
            }
            Packet::PeerJoinAttempt {
                target_id,
                metadata,
            } => {
                push_client_id(&mut buf, *target_id);
                push_string(&mut buf, metadata);
            }
            Packet::PeerJoinedRoom { peer_id }
            | Packet::PeerLeftRoom { peer_id }
            | Packet::DisconnectPeer { peer_id } => push_i32(&mut buf, *peer_id),
            Packet::GameData { from_peer, data } => {
                push_i32(&mut buf, *from_peer);
                buf.extend(data);
            }
            Packet::Error {
                error_code,
                error_message,
            } => {
                push_i32(&mut buf, error_code.as_i32());
                push_string(&mut buf, error_message);
            }
            Packet::Hello { nonce, seed } => {
                push_u64(&mut buf, *nonce);
                push_fixed_32(&mut buf, seed);
            }
            Packet::Cookie { nonce, token } => {
                push_u64(&mut buf, *nonce);
                push_fixed_32(&mut buf, token);
            }
            Packet::Ping { nonce } | Packet::Pong { nonce } => push_u64(&mut buf, *nonce),
            Packet::SendFailed {
                target_peer,
                reason,
            } => {
                push_i32(&mut buf, *target_peer);
                push_i32(&mut buf, reason.as_i32());
            }
        }

        debug_assert_eq!(buf.len(), encoded_len);
        Ok(buf)
    }

    fn encoded_len(&self) -> Result<usize, ProtocolError> {
        let len = match self {
            Packet::Authenticate {
                app_id, version, ..
            } => {
                check_field("app id", app_id.len(), MAX_APP_ID_BYTES)?;
                check_field("version", version.len(), MAX_VERSION_BYTES)?;
                1 + 8 + 32 + 32 + encoded_string_len(app_id) + encoded_string_len(version)
            }
            Packet::ClientAuthenticated | Packet::ForceDisconnect | Packet::Disconnect => 1,
            Packet::CreateRoom { metadata, .. } => {
                check_field("room metadata", metadata.len(), MAX_ROOM_METADATA_BYTES)?;
                1 + 4 + 4 + encoded_string_len(metadata)
            }
            Packet::ReqRooms { .. } => 1 + 8 + 8,
            Packet::GetRooms { rooms, .. } => {
                if rooms.len() > MAX_ROOMS_PER_PAGE {
                    return Err(ProtocolError::TooManyRooms {
                        actual: rooms.len(),
                        maximum: MAX_ROOMS_PER_PAGE,
                    });
                }
                let mut len = 1 + 4 + 8 + 8;
                for room in rooms {
                    check_field("room join code", room.join_code.len(), MAX_ROOM_CODE_BYTES)?;
                    check_field(
                        "room metadata",
                        room.metadata.len(),
                        MAX_ROOM_METADATA_BYTES,
                    )?;
                    len += encoded_string_len(&room.join_code) + encoded_string_len(&room.metadata);
                }
                len
            }
            Packet::UpdateRoom { room_id, metadata } => {
                check_field("room code", room_id.len(), MAX_ROOM_CODE_BYTES)?;
                check_field("room metadata", metadata.len(), MAX_ROOM_METADATA_BYTES)?;
                1 + encoded_string_len(room_id) + encoded_string_len(metadata)
            }
            Packet::ReqJoin { room_id, metadata } => {
                check_field("room code", room_id.len(), MAX_ROOM_CODE_BYTES)?;
                check_field("join metadata", metadata.len(), MAX_JOIN_METADATA_BYTES)?;
                1 + encoded_string_len(room_id) + encoded_string_len(metadata)
            }
            Packet::JoinRes { room_id, .. } => {
                check_field("room code", room_id.len(), MAX_ROOM_CODE_BYTES)?;
                1 + 8 + encoded_string_len(room_id) + 4
            }
            Packet::ConnectedToRoom { room_id, .. } => {
                check_field("room code", room_id.len(), MAX_ROOM_CODE_BYTES)?;
                1 + encoded_string_len(room_id) + 4
            }
            Packet::PeerJoinAttempt { metadata, .. } => {
                check_field("join metadata", metadata.len(), MAX_JOIN_METADATA_BYTES)?;
                1 + 8 + encoded_string_len(metadata)
            }
            Packet::PeerJoinedRoom { .. }
            | Packet::PeerLeftRoom { .. }
            | Packet::DisconnectPeer { .. } => 1 + 4,
            Packet::GameData { data, .. } => {
                check_field("game data", data.len(), MAX_GAME_DATA_BYTES)?;
                1 + 4 + data.len()
            }
            Packet::Error { error_message, .. } => {
                check_field(
                    "error message",
                    error_message.len(),
                    MAX_ERROR_MESSAGE_BYTES,
                )?;
                1 + 4 + encoded_string_len(error_message)
            }
            Packet::Hello { .. } | Packet::Cookie { .. } => 1 + 8 + 32,
            Packet::Ping { .. } | Packet::Pong { .. } => 1 + 8,
            Packet::SendFailed { .. } => 1 + 4 + 4,
        };

        if len > MAX_PROTOCOL_PACKET_BYTES {
            return Err(ProtocolError::PacketTooLarge {
                actual: len,
                maximum: MAX_PROTOCOL_PACKET_BYTES,
            });
        }
        Ok(len)
    }
}

const fn encoded_string_len(value: &str) -> usize {
    4 + value.len()
}

fn check_field(field: &'static str, actual: usize, maximum: usize) -> Result<(), ProtocolError> {
    if actual > maximum {
        return Err(ProtocolError::FieldTooLarge {
            field,
            actual,
            maximum,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_round_trips(packet: Packet) {
        let bytes = packet
            .to_bytes()
            .unwrap_or_else(|e| panic!("failed to encode {packet:?}: {e}"));
        let decoded = Packet::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("failed to decode {packet:?} from its own encoding: {e}"));

        assert_eq!(decoded, packet);
        assert_eq!(
            decoded.to_bytes().expect("decoded packet must re-encode"),
            bytes,
            "re-encoding a decoded packet produced different bytes for {packet:?}"
        );
    }

    fn authenticate(app_id: String, version: String) -> Packet {
        Packet::Authenticate {
            nonce: 7,
            seed: [1; 32],
            cookie: [2; 32],
            app_id,
            version,
        }
    }

    #[test]
    fn authenticate_round_trips() {
        assert_round_trips(authenticate("my-app".to_string(), "1.2.0_beta".to_string()));
    }

    #[test]
    fn create_room_round_trips() {
        assert_round_trips(Packet::CreateRoom {
            is_public: true,
            max_peers: 8,
            metadata: "hello".to_string(),
        });
        assert_round_trips(Packet::CreateRoom {
            is_public: false,
            max_peers: 64,
            metadata: String::new(),
        });
    }

    #[test]
    fn get_rooms_round_trips_with_join_code_field() {
        // Regression test: `RoomInfo` previously had its wire-serialized
        // field named `join_code` on the relay server and `id` on the
        // Godot client, even though both encoded/decoded it identically.
        // This test exercises the shared type both sides now use.
        assert_round_trips(Packet::GetRooms {
            rooms: vec![
                RoomInfo {
                    join_code: "ABCDE".to_string(),
                    metadata: "meta".to_string(),
                },
                RoomInfo {
                    join_code: "12345".to_string(),
                    metadata: String::new(),
                },
            ],
            next_cursor: 42,
            snapshot_end: 100,
        });
    }

    #[test]
    fn join_res_round_trips_client_id() {
        assert_round_trips(Packet::JoinRes {
            target_id: ClientId::new(42),
            room_id: "ABCDE".to_string(),
            allowed: true,
        });
    }

    #[test]
    fn peer_join_attempt_round_trips_client_id() {
        assert_round_trips(Packet::PeerJoinAttempt {
            target_id: ClientId::new(9_999_999_999),
            metadata: "join metadata".to_string(),
        });
    }

    #[test]
    fn game_data_round_trips_arbitrary_bytes() {
        assert_round_trips(Packet::GameData {
            from_peer: -7,
            data: vec![0, 1, 2, 255, 254, 253],
        });
    }

    #[test]
    fn all_variants_have_matching_kind() {
        let samples = vec![
            authenticate(String::new(), String::new()),
            Packet::ClientAuthenticated,
            Packet::CreateRoom {
                is_public: false,
                max_peers: 8,
                metadata: String::new(),
            },
            Packet::ReqRooms {
                cursor: 0,
                snapshot_end: 0,
            },
            Packet::GetRooms {
                rooms: vec![],
                next_cursor: 0,
                snapshot_end: 1,
            },
            Packet::UpdateRoom {
                room_id: String::new(),
                metadata: String::new(),
            },
            Packet::ReqJoin {
                room_id: String::new(),
                metadata: String::new(),
            },
            Packet::JoinRes {
                target_id: ClientId::new(1),
                room_id: String::new(),
                allowed: false,
            },
            Packet::ConnectedToRoom {
                room_id: String::new(),
                peer_id: 0,
            },
            Packet::PeerJoinAttempt {
                target_id: ClientId::new(1),
                metadata: String::new(),
            },
            Packet::PeerJoinedRoom { peer_id: 0 },
            Packet::PeerLeftRoom { peer_id: 0 },
            Packet::GameData {
                from_peer: 0,
                data: vec![],
            },
            Packet::ForceDisconnect,
            Packet::Error {
                error_code: ErrorCode::Internal,
                error_message: String::new(),
            },
            Packet::Hello {
                nonce: 1,
                seed: [3; 32],
            },
            Packet::Cookie {
                nonce: 1,
                token: [4; 32],
            },
            Packet::Ping { nonce: 2 },
            Packet::Pong { nonce: 2 },
            Packet::Disconnect,
            Packet::DisconnectPeer { peer_id: 3 },
            Packet::SendFailed {
                target_peer: -4,
                reason: ErrorCode::Conflict,
            },
        ];

        for packet in samples {
            let bytes = packet.to_bytes().expect("sample packet must encode");
            assert_eq!(bytes[0], packet.kind().as_u8());
            assert_round_trips(packet);
        }
    }

    #[test]
    fn empty_bytes_is_rejected() {
        assert!(matches!(
            Packet::from_bytes(&[]),
            Err(ProtocolError::EmptyPacket)
        ));
    }

    #[test]
    fn unknown_packet_id_is_rejected() {
        assert!(matches!(
            Packet::from_bytes(&[255]),
            Err(ProtocolError::UnknownPacketType(255))
        ));
    }

    #[test]
    fn truncated_create_room_metadata_is_rejected() {
        // Regression test: `CreateRoom` decoding previously swallowed any
        // error from reading the metadata string via `unwrap_or_default`,
        // silently succeeding with empty metadata for a truncated/corrupt
        // packet instead of returning an error like every other variant.
        let mut bytes = vec![PacketKind::CreateRoom.as_u8()];
        push_bool(&mut bytes, true);
        // Claim a metadata string of length 10, but provide no bytes for
        // it at all — this should be rejected, not decoded as `""`.
        push_i32(&mut bytes, 10);

        assert!(matches!(
            Packet::from_bytes(&bytes),
            Err(ProtocolError::NotEnoughBytes(_))
        ));
    }

    #[test]
    fn hostile_lengths_and_packet_sizes_are_rejected() {
        let mut oversized_count = vec![PacketKind::GetRooms.as_u8()];
        push_i32(&mut oversized_count, i32::MAX);
        assert!(matches!(
            Packet::from_bytes(&oversized_count),
            Err(ProtocolError::TooManyRooms { .. })
        ));

        let mut oversized_string = vec![PacketKind::Authenticate.as_u8()];
        push_u64(&mut oversized_string, 1);
        push_fixed_32(&mut oversized_string, &[0; 32]);
        push_fixed_32(&mut oversized_string, &[0; 32]);
        push_i32(&mut oversized_string, (MAX_APP_ID_BYTES + 1) as i32);
        assert!(matches!(
            Packet::from_bytes(&oversized_string),
            Err(ProtocolError::FieldTooLarge {
                field: "app id",
                ..
            })
        ));

        let oversized_packet = vec![0; MAX_PROTOCOL_PACKET_BYTES + 1];
        assert!(matches!(
            Packet::from_bytes(&oversized_packet),
            Err(ProtocolError::PacketTooLarge { .. })
        ));
    }

    #[test]
    fn invalid_boolean_and_trailing_control_bytes_are_rejected() {
        let mut invalid_boolean = vec![PacketKind::CreateRoom.as_u8()];
        push_i32(&mut invalid_boolean, 2);
        push_string(&mut invalid_boolean, "");
        assert!(matches!(
            Packet::from_bytes(&invalid_boolean),
            Err(ProtocolError::InvalidBoolean(2))
        ));

        assert!(matches!(
            Packet::from_bytes(&[PacketKind::ClientAuthenticated.as_u8(), 0]),
            Err(ProtocolError::TrailingBytes(1))
        ));
    }

    #[test]
    fn encoding_rejects_every_field_at_max_plus_one() {
        let cases = [
            authenticate("a".repeat(MAX_APP_ID_BYTES + 1), String::new()),
            authenticate(String::new(), "v".repeat(MAX_VERSION_BYTES + 1)),
            Packet::UpdateRoom {
                room_id: "r".repeat(MAX_ROOM_CODE_BYTES + 1),
                metadata: String::new(),
            },
            Packet::CreateRoom {
                is_public: true,
                max_peers: 8,
                metadata: "m".repeat(MAX_ROOM_METADATA_BYTES + 1),
            },
            Packet::ReqJoin {
                room_id: String::new(),
                metadata: "j".repeat(MAX_JOIN_METADATA_BYTES + 1),
            },
            Packet::Error {
                error_code: ErrorCode::InvalidRequest,
                error_message: "e".repeat(MAX_ERROR_MESSAGE_BYTES + 1),
            },
            Packet::GameData {
                from_peer: 0,
                data: vec![0; MAX_GAME_DATA_BYTES + 1],
            },
        ];

        for packet in cases {
            assert!(matches!(
                packet.to_bytes(),
                Err(ProtocolError::FieldTooLarge { .. })
            ));
        }

        let too_many_rooms = Packet::GetRooms {
            rooms: vec![
                RoomInfo {
                    join_code: String::new(),
                    metadata: String::new()
                };
                MAX_ROOMS_PER_PAGE + 1
            ],
            next_cursor: 0,
            snapshot_end: 1,
        };
        assert!(matches!(
            too_many_rooms.to_bytes(),
            Err(ProtocolError::TooManyRooms { .. })
        ));
    }

    #[test]
    fn encoding_accepts_exact_field_maxima() {
        let cases = [
            authenticate("a".repeat(MAX_APP_ID_BYTES), "v".repeat(MAX_VERSION_BYTES)),
            Packet::UpdateRoom {
                room_id: "r".repeat(MAX_ROOM_CODE_BYTES),
                metadata: "m".repeat(MAX_ROOM_METADATA_BYTES),
            },
            Packet::ReqJoin {
                room_id: "r".repeat(MAX_ROOM_CODE_BYTES),
                metadata: "j".repeat(MAX_JOIN_METADATA_BYTES),
            },
            Packet::Error {
                error_code: ErrorCode::InvalidRequest,
                error_message: "e".repeat(MAX_ERROR_MESSAGE_BYTES),
            },
            Packet::GameData {
                from_peer: i32::MIN,
                data: vec![0xff; MAX_GAME_DATA_BYTES],
            },
        ];

        for packet in cases {
            assert_round_trips(packet);
        }
    }

    #[test]
    fn hello_is_not_smaller_than_cookie() {
        let hello = Packet::Hello {
            nonce: 1,
            seed: [0; 32],
        }
        .to_bytes()
        .expect("hello must encode");
        let cookie = Packet::Cookie {
            nonce: 1,
            token: [0; 32],
        }
        .to_bytes()
        .expect("cookie must encode");
        assert!(hello.len() >= cookie.len());
    }
}
