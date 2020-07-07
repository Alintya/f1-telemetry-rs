use super::{f1_2019, f1_2020};
use car_setup::PacketCarSetupData;
use car_status::PacketCarStatusData;
use car_telemetry::PacketCarTelemetryData;
use event::PacketEventData;
use header::PacketHeader;
use lap::PacketLapData;
use motion::PacketMotionData;
use participants::PacketParticipantsData;
use session::PacketSessionData;

pub mod car_setup;
pub mod car_status;
pub mod car_telemetry;
pub mod event;
pub mod generic;
pub mod header;
pub mod lap;
pub mod motion;
pub mod participants;
pub mod session;

#[derive(Debug)]
pub struct UnpackError(pub String);

#[derive(Debug)]
pub enum Packet {
    Motion(PacketMotionData),
    Session(PacketSessionData),
    Lap(PacketLapData),
    Event(PacketEventData),
    Participants(PacketParticipantsData),
    CarSetups(PacketCarSetupData),
    CarTelemetry(PacketCarTelemetryData),
    CarStatus(PacketCarStatusData),
}

impl Packet {
    pub fn header(&self) -> &PacketHeader {
        match self {
            Packet::Motion(p) => p.header(),
            Packet::Session(p) => p.header(),
            Packet::Lap(p) => p.header(),
            Packet::Event(p) => p.header(),
            Packet::Participants(p) => p.header(),
            Packet::CarSetups(p) => p.header(),
            Packet::CarTelemetry(p) => p.header(),
            Packet::CarStatus(p) => p.header(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum PacketType {
    Motion,
    Session,
    LapData,
    Event,
    Participants,
    CarSetups,
    CarTelemetry,
    CarStatus,
    FinalClassification,
    LobbyInfo,
}

pub(crate) fn parse_packet(size: usize, packet: &[u8]) -> Result<Packet, UnpackError> {
    let packet_format = parse_version(packet);

    match packet_format {
        2019 => Ok(f1_2019::parse_packet(size, packet)?),
        2020 => Ok(f1_2020::parse_packet(size, packet)?),
        _ => Err(UnpackError(format!(
            "Invalid packet: unknown format ({})",
            packet_format
        ))),
    }
}

fn parse_version(packet: &[u8]) -> u16 {
    packet[0] as u16 | ((packet[1] as u16) << 8)
}
