use crate::models::TelemetryInfo;
use crate::render::{CarRenderer, LapRenderer, MainRenderer, Renderer, TrackRenderer};
use f1_telemetry::packet::car_telemetry::PacketCarTelemetryData;
use f1_telemetry::packet::event::PacketEventData;
use f1_telemetry::packet::lap::{PacketLapData, PitStatus};
use f1_telemetry::packet::participants::PacketParticipantsData;
use f1_telemetry::packet::session::PacketSessionData;
use f1_telemetry::Stream;
use models::{EventInfo, LapInfo, SessionInfo};
use std::thread::sleep;
use std::time::Duration;
use ui::{Ui, Window};

mod models;
mod render;
mod ui;

fn main() {
    let stream = Stream::new("0.0.0.0:20777").expect("Unable to bind socket");
    println!("Listening on {}", stream.socket().local_addr().unwrap());

    let mut ui = Ui::init();

    loop {
        match stream.next() {
            Ok(p) => match p {
                Some(p) => {
                    let mr: MainRenderer = Renderer::new();
                    mr.render(&mut ui, &p);

                    match ui.active_window {
                        Window::Lap => {
                            let r: LapRenderer = Renderer::new();
                            r.render(&mut ui, &p)
                        }
                        Window::Track => {
                            let r: TrackRenderer = Renderer::new();
                            r.render(&mut ui, &p)
                        }
                        Window::Car => {
                            let r: CarRenderer = Renderer::new();
                            r.render(&mut ui, &p)
                        }
                    };
                }
                None => sleep(Duration::from_millis(5)),
            },
            Err(_e) => {
                panic!("{:?}", _e);
            }
        }

        let ch = ncurses::get_wch();
        if let Some(ch) = ch {
            match ch {
                ncurses::WchResult::Char(49) => {
                    // 1
                    ui.switch_window(Window::Lap);
                }
                ncurses::WchResult::Char(50) => {
                    // 2
                    ui.switch_window(Window::Car);
                }
                ncurses::WchResult::Char(51) => {
                    // 3
                    ui.switch_window(Window::Track);
                }
                ncurses::WchResult::Char(113) => {
                    // q
                    break;
                }
                // ncurses::WchResult::Char(c) => {
                //     ncurses::mvaddstr(0, 0, format!("Pressed Char: {}", c).as_str());
                // }
                // ncurses::WchResult::KeyCode(c) => {
                //     ncurses::mvaddstr(0, 0, format!("Pressed Key: {}", c).as_str());
                //     ncurses::clrtoeol();
                // }
                _ => {}
            }
        }
    }

    ui.destroy();
}

fn parse_session_data(session: &PacketSessionData, current_lap: u8) -> SessionInfo {
    SessionInfo {
        session_name: session.session_type().name(),
        track_name: session.track().name(),
        elapsed_time: session.session_duration() - session.session_time_left(),
        duration: session.session_duration(),
        current_lap,
        number_of_laps: session.total_laps(),
    }
}

fn parse_lap_data<'a>(
    lap_data: &'a PacketLapData,
    participants: &'a Option<PacketParticipantsData>,
) -> Option<Vec<LapInfo<'a>>> {
    if participants.is_none() {
        return None;
    }

    let participants = participants.as_ref().unwrap().participants();

    let mut lap_info = Vec::with_capacity(lap_data.lap_data().len());

    for (i, ld) in lap_data.lap_data().iter().enumerate() {
        let li = LapInfo {
            position: ld.car_position(),
            name: participants[i].name(),
            driver: participants[i].driver(),
            team: participants[i].team(),
            current_lap_time: ld.current_lap_time(),
            last_lap_time: ld.last_lap_time(),
            best_lap_time: ld.best_lap_time(),
            status: ld.result_status(),
            in_pit: ld.pit_status() != PitStatus::None,
            lap_invalid: ld.current_lap_invalid(),
            penalties: ld.penalties(),
            lap_distance: ld.lap_distance(),
            total_distance: ld.total_distance(),
        };

        lap_info.push(li);
    }

    Some(lap_info)
}

fn parse_event_data<'a>(
    event_data: &'a PacketEventData,
    participants: &'a Option<PacketParticipantsData>,
) -> Option<EventInfo<'a>> {
    if event_data.vehicle_idx().is_some() && participants.is_none() {
        return None;
    }

    let description = event_data.event().description();
    let driver_name = match event_data.vehicle_idx() {
        Some(idx) => {
            let name = participants.as_ref().unwrap().participants()[idx as usize].name();
            Some(name.as_str())
        }
        None => None,
    };

    Some(EventInfo {
        timestamp: event_data.header().session_time(),
        description,
        driver_name,
        lap_time: event_data.lap_time(),
    })
}

fn parse_telemetry_data(telemetry_data: &PacketCarTelemetryData) -> Option<TelemetryInfo> {
    let player_index = telemetry_data.header().player_car_index();
    let telemetry_data = &telemetry_data.car_telemetry_data()[player_index as usize];

    Some(TelemetryInfo {
        speed: telemetry_data.speed(),
        throttle: telemetry_data.throttle(),
        brake: telemetry_data.brake(),
        gear: telemetry_data.gear(),
        engine_rpm: telemetry_data.engine_rpm(),
        drs: telemetry_data.drs(),
        rev_lights_percent: telemetry_data.rev_lights_percent(),
        engine_temperature: telemetry_data.engine_temperature(),
    })
}

fn get_current_lap(lap_data: &PacketLapData) -> u8 {
    lap_data
        .lap_data()
        .iter()
        .map(|l| l.current_lap_num())
        .max()
        .unwrap_or(0)
}
