use crate::packet::header::PacketHeader;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::BufRead;

impl PacketHeader {
    pub(crate) fn size() -> usize {
        24
    }
}

pub(crate) fn parse_header<T: BufRead>(reader: &mut T) -> PacketHeader {
    let packet_format = reader.read_u16::<LittleEndian>().unwrap();
    let game_major_version = reader.read_u8().unwrap();
    let game_minor_version = reader.read_u8().unwrap();
    let packet_version = reader.read_u8().unwrap();
    let packet_id = reader.read_u8().unwrap();
    let session_uid = reader.read_u64::<LittleEndian>().unwrap();
    let session_time = reader.read_f32::<LittleEndian>().unwrap();
    let frame_identifier = reader.read_u32::<LittleEndian>().unwrap();
    let player_car_index = reader.read_u8().unwrap();

    PacketHeader::new(
        packet_format,
        game_major_version,
        game_minor_version,
        packet_version,
        packet_id,
        session_uid,
        session_time,
        frame_identifier,
        player_car_index,
        255,
    )
}
