use crate::error::ProtocolError;
use crate::packet::RoomInfo;

pub fn read_bool(bytes: &[u8]) -> Result<(bool, &[u8]), ProtocolError> {
    let (value, rest) = read_i32(bytes)?;
    match value {
        0 => Ok((false, rest)),
        1 => Ok((true, rest)),
        other => Err(ProtocolError::InvalidBoolean(other)),
    }
}

pub fn read_i32(bytes: &[u8]) -> Result<(i32, &[u8]), ProtocolError> {
    if bytes.len() < 4 {
        return Err(ProtocolError::NotEnoughBytes(format!(
            "for i32 (need 4 bytes, have {})",
            bytes.len()
        )));
    }
    let value = i32::from_be_bytes(bytes[..4].try_into()?);
    Ok((value, &bytes[4..]))
}

pub fn read_u64(bytes: &[u8]) -> Result<(u64, &[u8]), ProtocolError> {
    if bytes.len() < 8 {
        return Err(ProtocolError::NotEnoughBytes(format!(
            "for u64 (need 8 bytes, have {})",
            bytes.len()
        )));
    }

    let value = u64::from_be_bytes(bytes[..8].try_into()?);
    Ok((value, &bytes[8..]))
}

pub fn read_fixed_32(bytes: &[u8]) -> Result<([u8; 32], &[u8]), ProtocolError> {
    if bytes.len() < 32 {
        return Err(ProtocolError::NotEnoughBytes(format!(
            "for 32-byte value (need 32 bytes, have {})",
            bytes.len()
        )));
    }
    let value = bytes[..32].try_into()?;
    Ok((value, &bytes[32..]))
}

pub fn read_string<'a>(
    bytes: &'a [u8],
    maximum: usize,
    field: &'static str,
) -> Result<(String, &'a [u8]), ProtocolError> {
    let (len, rest) = read_i32(bytes)?;

    if len < 0 {
        return Err(ProtocolError::NegativeVectorLength);
    }

    let len = len as usize;
    if len > maximum {
        return Err(ProtocolError::FieldTooLarge {
            field,
            actual: len,
            maximum,
        });
    }
    if rest.len() < len {
        return Err(ProtocolError::NotEnoughBytes(format!(
            "for string (need {len} bytes, have {})",
            rest.len()
        )));
    }

    let string_bytes = &rest[..len];
    let remaining = &rest[len..];

    Ok((String::from_utf8(string_bytes.to_vec())?, remaining))
}

pub fn push_string(buf: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    buf.extend((bytes.len() as i32).to_be_bytes());
    buf.extend(bytes);
}

pub fn push_bool(buf: &mut Vec<u8>, value: bool) {
    push_i32(buf, i32::from(value));
}

pub fn push_i32(buf: &mut Vec<u8>, value: i32) {
    buf.extend(value.to_be_bytes());
}

pub fn push_u64(buf: &mut Vec<u8>, value: u64) {
    buf.extend(value.to_be_bytes());
}

pub fn push_fixed_32(buf: &mut Vec<u8>, value: &[u8; 32]) {
    buf.extend(value);
}

pub fn read_room_info(bytes: &[u8]) -> Result<(RoomInfo, &[u8]), ProtocolError> {
    let (join_code, r) = read_string(bytes, crate::packet::MAX_ROOM_CODE_BYTES, "room join code")?;
    let (metadata, r) = read_string(r, crate::packet::MAX_ROOM_METADATA_BYTES, "room metadata")?;

    Ok((
        RoomInfo {
            join_code,
            metadata,
        },
        r,
    ))
}

pub fn read_vec_room_info(bytes: &[u8]) -> Result<(Vec<RoomInfo>, &[u8]), ProtocolError> {
    let (len, mut rest) = read_i32(bytes)?;

    if len < 0 {
        return Err(ProtocolError::NegativeVectorLength);
    }

    let len = len as usize;
    let structural_maximum = rest.len() / 8;
    let maximum = crate::packet::MAX_ROOMS_PER_PAGE.min(structural_maximum);
    if len > maximum {
        return Err(ProtocolError::TooManyRooms {
            actual: len,
            maximum,
        });
    }

    let mut rooms = Vec::with_capacity(len);
    for _ in 0..len {
        let (room, remaining) = read_room_info(rest)?;
        rooms.push(room);
        rest = remaining;
    }

    Ok((rooms, rest))
}

pub fn push_vec_room_info(buf: &mut Vec<u8>, rooms: &[RoomInfo]) {
    push_i32(buf, rooms.len() as i32);
    for room in rooms {
        push_string(buf, &room.join_code);
        push_string(buf, &room.metadata);
    }
}
