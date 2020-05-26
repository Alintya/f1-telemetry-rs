use std::convert::TryFrom;
use std::io::Cursor;
use std::mem;

use car_status::PacketCarStatusData;
use car_telemetry::PacketCarTelemetryData;
use event::PacketEventData;
use header::PacketHeader;
use lap::PacketLapData;
use participants::PacketParticipantsData;
use session::PacketSessionData;

pub mod car_status;
pub mod car_telemetry;
pub mod event;
pub mod generic;
pub mod header;
pub mod lap;
pub mod participants;
pub mod session;

//struct UnpackError(&'static str);
#[derive(Debug)]
pub struct UnpackError(pub String);

pub enum Packet {
    Session(PacketSessionData),
    Lap(PacketLapData),
    Event(PacketEventData),
    Participants(PacketParticipantsData),
    CarTelemetry(PacketCarTelemetryData),
    CarStatus(PacketCarStatusData),
}

#[derive(Debug)]
enum PacketType {
    Motion,
    Session,
    LapData,
    Event,
    Participants,
    CarSetups,
    CarTelemetry,
    CarStatus,
}

impl TryFrom<u8> for PacketType {
    type Error = UnpackError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PacketType::Motion),
            1 => Ok(PacketType::Session),
            2 => Ok(PacketType::LapData),
            3 => Ok(PacketType::Event),
            4 => Ok(PacketType::Participants),
            5 => Ok(PacketType::CarSetups),
            6 => Ok(PacketType::CarTelemetry),
            7 => Ok(PacketType::CarStatus),
            _ => Err(UnpackError(format!("Invalid PacketType: {}", value))),
        }
    }
}

pub fn parse_packet(size: usize, packet: &[u8]) -> Result<Packet, UnpackError> {
    let header_size = mem::size_of::<PacketHeader>();

    if size < header_size {
        return Err(UnpackError(format!(
            "Invalid packet: too small ({} bytes)",
            size
        )));
    }

    let mut cursor = Cursor::new(packet);
    let header = PacketHeader::new(&mut cursor);

    let packet_id: PacketType = PacketType::try_from(*header.packet_id())?;

    match packet_id {
        //PacketType::Motion => {
        //}
        PacketType::Session => {
            let packet = PacketSessionData::new(&mut cursor, header)?;

            Ok(Packet::Session(packet))
        }
        PacketType::LapData => {
            let packet = PacketLapData::new(&mut cursor, header)?;

            Ok(Packet::Lap(packet))
        }
        PacketType::Event => {
            let packet = PacketEventData::new(&mut cursor, header)?;

            Ok(Packet::Event(packet))
        }
        PacketType::Participants => {
            let packet = PacketParticipantsData::new(&mut cursor, header)?;

            Ok(Packet::Participants(packet))
        }
        //PacketType::CarSetups => {
        //}
        PacketType::CarTelemetry => {
            let packet = PacketCarTelemetryData::new(&mut cursor, header)?;

            Ok(Packet::CarTelemetry(packet))
        }
        PacketType::CarStatus => {
            let packet = PacketCarStatusData::new(&mut cursor, header)?;

            Ok(Packet::CarStatus(packet))
        }
        _ => Err(UnpackError(format!(
            "Unpacking not implemented for {:?}",
            packet_id
        ))),
    }
}
