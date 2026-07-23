use std::fs;
use std::path::PathBuf;

use nodetunnel_protocol::packet::{
    MAX_PROTOCOL_PACKET_BYTES, Packet, RoomInfo,
};
use nodetunnel_protocol::{ClientId, ErrorCode, PacketKind};

fn main() {
    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/from_bytes");
    fs::create_dir_all(&corpus).expect("create protocol corpus directory");

    let packets = [
        Packet::Authenticate {
            nonce: 1,
            seed: [1; 32],
            cookie: [2; 32],
            app_id: "app".into(),
            version: "1.2.0_beta".into(),
        },
        Packet::ClientAuthenticated,
        Packet::CreateRoom { is_public: true, max_peers: 8, metadata: "room".into() },
        Packet::ReqJoin { room_id: "ABCDE".into(), metadata: "join".into() },
        Packet::ConnectedToRoom { room_id: "ABCDE".into(), peer_id: 1 },
        Packet::PeerJoinedRoom { peer_id: 2 },
        Packet::PeerLeftRoom { peer_id: 2 },
        Packet::GameData { from_peer: -1, data: vec![0, 1, 2, 255] },
        Packet::ForceDisconnect,
        Packet::Error { error_code: ErrorCode::InvalidRequest, error_message: "invalid".into() },
        Packet::ReqRooms { cursor: 0, snapshot_end: 0 },
        Packet::GetRooms {
            rooms: vec![RoomInfo { join_code: "ABCDE".into(), metadata: "room".into() }],
            next_cursor: 0,
            snapshot_end: 2,
        },
        Packet::UpdateRoom { room_id: "ABCDE".into(), metadata: "updated".into() },
        Packet::JoinRes { target_id: ClientId::new(7), room_id: "ABCDE".into(), allowed: true },
        Packet::PeerJoinAttempt { target_id: ClientId::new(7), metadata: "join".into() },
        Packet::Hello { nonce: 9, seed: [3; 32] },
        Packet::Cookie { nonce: 9, token: [4; 32] },
        Packet::Ping { nonce: 10 },
        Packet::Pong { nonce: 10 },
        Packet::Disconnect,
        Packet::DisconnectPeer { peer_id: 2 },
        Packet::SendFailed { target_peer: -2, reason: ErrorCode::Conflict },
    ];

    for (index, packet) in packets.into_iter().enumerate() {
        fs::write(
            corpus.join(format!("valid-{index:02}-{:02}", packet.kind().as_u8())),
            packet.to_bytes().expect("valid seed packet must encode"),
        )
        .expect("write valid protocol corpus entry");
    }

    let malformed = [
        ("empty", Vec::new()),
        ("truncated", vec![PacketKind::Hello.as_u8(), 0]),
        ("max-size", vec![PacketKind::GameData.as_u8(); MAX_PROTOCOL_PACKET_BYTES]),
        ("wrong-id", vec![PacketKind::ErrorPacket.as_u8(), 0, 0, 0, 42, 0, 0, 0, 0]),
        ("unknown-type", vec![255]),
        (
            "oversized-count",
            vec![PacketKind::GetRooms.as_u8(), 0x7f, 0xff, 0xff, 0xff],
        ),
    ];
    for (name, bytes) in malformed {
        fs::write(corpus.join(name), bytes).expect("write malformed protocol corpus entry");
    }
}
