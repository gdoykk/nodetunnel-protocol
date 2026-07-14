# nodetunnel-protocol

Shared wire protocol for [NodeTunnel](..), used by both
[`relay-server`](../relay-server) and [`godot-plugin`](../godot-plugin).
This crate exists specifically to keep the relay and the client from
drifting apart on packet layout — as happened previously with `RoomInfo`'s
field being named `join_code` on one side and `id` on the other while
encoding/decoding identically. Both crates depend on this one rather than
each maintaining their own copy of the wire format.

## Wire format

- Big-endian for all multi-byte values.
- Strings are length-prefixed: an `i32` byte length followed by UTF-8
  bytes.
- Booleans and integers are encoded as `i32` unless a field has a more
  specific type (e.g. `ClientId` is `u64`).
- See `src/serialize.rs` for the low-level read/push helpers primitives
  and `RoomInfo` are built from.

## Key files

- `src/ids.rs` — `PacketKind`, a real `#[repr(u8)]` enum (not loose `u8`
  constants) identifying the kind of a `Packet` on the wire. A `TryFrom<u8>`
  impl rejects unknown discriminants explicitly rather than falling back to
  a default variant, and the enum keeps the discriminant-to-variant mapping
  in exactly one place.
- `src/packet.rs` — the `Packet` enum (one variant per `PacketKind`) plus
  `from_bytes`/`to_bytes` for (de)serializing a full packet. Has inline
  `#[cfg(test)]` unit tests covering encode/decode round-trips.
- `src/serialize.rs` — big-endian read/push helpers for primitives
  (`u64`, `i32`, strings, etc.) and `RoomInfo`.
- `src/client_id.rs` — `ClientId(u64)`, a newtype for a client's
  transport-level session ID with the relay. Deliberately distinct from
  the per-room Godot `MultiplayerPeer` id (an `i32` assigned when a client
  joins a room): `ClientId` is stable for the life of the UDP session,
  the Godot peer id is scoped to a single room. Wrapping it prevents
  accidentally mixing it up with room/app/peer ids in function signatures
  that take several integer parameters.
- `src/error.rs` — `ProtocolError` (via `thiserror`), covering decode
  failures such as unknown packet types.
- `src/version.rs` — `PROTOCOL_VERSION`, the version string sent during
  authentication and checked by the relay's `ALLOWED_VERSIONS` config.

## Adding a new packet type

1. Add a variant to `PacketKind` in `src/ids.rs` with an explicit `u8`
   discriminant, and the corresponding `TryFrom<u8>` arm.
2. Add a matching variant to `Packet` in `src/packet.rs`.
3. Implement both the `from_bytes` and `to_bytes` arms for it. The crate
   favors exhaustive `match` over fallback arms so the compiler catches
   any missing case.
4. Add an encode/decode round-trip test alongside the existing
   `#[cfg(test)]` tests in `src/packet.rs`.
5. Update call sites in both `relay-server` and `godot-plugin` — this
   crate is a path/git dependency for both, so a packet layout change is
   a breaking change for both.
6. Bump `PROTOCOL_VERSION` in `src/version.rs` if the change is
   wire-incompatible with clients/relays running the previous version.

## Building and testing

```
cargo build
cargo test
cargo clippy
cargo fmt
```

## Notes

- No root workspace: build and test this crate from within `protocol/`.
- Prefer newtypes (like `ClientId`) over bare integers whenever a value
  has distinct semantics from other IDs in scope.
