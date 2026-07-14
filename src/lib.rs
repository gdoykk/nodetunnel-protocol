//! Shared wire protocol for NodeTunnel, used by both the relay server and the
//! Godot client plugin. Keeping this in one crate prevents the two
//! implementations from drifting apart (as happened previously with
//! `RoomInfo`'s field being named `join_code` on one side and `id` on the
//! other while encoding/decoding identically).

pub mod client_id;
mod ids;
pub mod packet;
mod serialize;
pub mod version;
pub mod error;

pub use client_id::ClientId;
