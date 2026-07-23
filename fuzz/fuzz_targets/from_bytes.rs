#![no_main]

use libfuzzer_sys::fuzz_target;
use nodetunnel_protocol::packet::Packet;

fuzz_target!(|data: &[u8]| {
    if let Ok(packet) = Packet::from_bytes(data) {
        if !matches!(packet, Packet::GameData { .. }) {
            let encoded = packet.to_bytes().expect("a decoded packet must always re-encode");
            let decoded = Packet::from_bytes(&encoded)
                .expect("a successfully encoded packet must always decode");
            assert_eq!(decoded, packet);
        }
    }
});
