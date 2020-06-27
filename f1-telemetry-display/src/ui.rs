use crate::models::{EventInfo, LapInfo, SessionInfo, TelemetryInfo};
use f1_telemetry::packet::lap::ResultStatus;
use f1_telemetry::packet::participants::Team;
use ncurses::*;
use std::collections::BTreeMap;
use std::f32::INFINITY;

mod fmt;

const MIN_WIDTH: i32 = 132;
const MIN_HEIGHT: i32 = 35;
const SESSION_Y_OFFSET: i32 = 1;
const WINDOW_Y_OFFSET: i32 = 5;
const LEFT_BORDER_X_OFFSET: i32 = 2;
const CURRENT_CAR_DATA_Y_OFFSET: i32 = 24;

pub enum Window {
    Lap,
    Car,
    Track,
}

pub struct Ui {
    mwnd: WINDOW,
    dashboard_wnd: WINDOW,
    car_wnd: WINDOW,
    track_wnd: WINDOW,
    active_wnd: WINDOW,
}

impl Ui {
    pub fn init() -> Ui {
        setlocale(ncurses::LcCategory::all, "");

        let mwnd = initscr();

        let w = getmaxx(mwnd);
        let h = getmaxy(mwnd);

        if w < MIN_WIDTH || h < MIN_HEIGHT {
            panic!(format!(
                "Terminal must be at least {}x{}. Current size: {}x{}",
                MIN_WIDTH, MIN_HEIGHT, w, h
            ));
        }

        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        cbreak();
        noecho();
        keypad(mwnd, true);
        timeout(0);
        fmt::init_colors();

        refresh();

        let win_w = w - 2;
        let win_h = h - WINDOW_Y_OFFSET - 2;

        let dashboard_wnd = Ui::create_win(win_h, win_w, WINDOW_Y_OFFSET, 1, Some("Dashboard"));
        let car_wnd = Ui::create_win(win_h, win_w, WINDOW_Y_OFFSET, 1, Some("Car Status"));
        let track_wnd = Ui::create_win(win_h, win_w, WINDOW_Y_OFFSET, 1, Some("Track Status"));

        let active_wnd = dashboard_wnd;
        wrefresh(active_wnd);

        Ui {
            mwnd,
            dashboard_wnd,
            car_wnd,
            track_wnd,
            active_wnd,
        }
    }

    pub fn destroy(&self) {
        endwin();
    }

    pub fn switch_window(&mut self, window: Window) {
        let neww = match window {
            Window::Lap => self.dashboard_wnd,
            Window::Car => self.car_wnd,
            Window::Track => self.track_wnd,
        };

        if neww == self.active_wnd {
            return;
        }

        redrawwin(neww);

        self.active_wnd = neww;
        self.refresh();
    }

    fn refresh(&self) {
        wrefresh(self.active_wnd);
    }

    fn create_win(h: i32, w: i32, y: i32, x: i32, title: Option<&str>) -> WINDOW {
        let wnd = newwin(h, w, y, x);
        box_(wnd, 0, 0);

        if let Some(title) = title {
            mvwaddstr(wnd, 0, 2, &format!(" {} ", title));
        };

        wnd
    }

    pub fn print_session_info(&self, sinfo: &SessionInfo) {
        let session_name = &format!("{} - {}", sinfo.session_name, sinfo.track_name);
        let lap_info = &format!("Lap {} of {}", sinfo.current_lap, sinfo.number_of_laps);
        let session_time = &format!(
            "{} / {}",
            fmt::format_time(sinfo.elapsed_time),
            fmt::format_time(sinfo.duration)
        );

        addstr_center(self.mwnd, SESSION_Y_OFFSET, session_name);
        addstr_center(self.mwnd, SESSION_Y_OFFSET + 1, lap_info);
        addstr_center(self.mwnd, SESSION_Y_OFFSET + 2, session_time);
    }

    pub fn print_lap_info(&self, lap_info: &[LapInfo]) {
        let wnd = self.dashboard_wnd;

        fmt::wset_bold(wnd);

        let header =
            "  P. NAME                 | CURRENT LAP  | LAST LAP     | BEST LAP     | STATUS";

        mvwaddstr(wnd, 1, LEFT_BORDER_X_OFFSET, header);

        for li in lap_info {
            let pos = match li.status {
                ResultStatus::Retired => String::from("RET"),
                ResultStatus::NotClassified => String::from("N/C"),
                ResultStatus::Disqualified => String::from("DSQ"),
                _ => format!("{:3}", li.position),
            };
            let penalties = if li.penalties > 0 {
                format!("+{:2}s", li.penalties)
            } else {
                "    ".to_string()
            };

            let s = format!(
                "{}. {:20} | {} | {} | {} | {}{}{} ",
                pos,
                fmt::format_driver_name(li.name, li.driver),
                fmt::format_time_ms(li.current_lap_time),
                fmt::format_time_ms(li.last_lap_time),
                fmt::format_time_ms(li.best_lap_time),
                if li.in_pit { "P" } else { " " },
                if li.lap_invalid { "!" } else { " " },
                penalties,
            );

            fmt::set_team_color(wnd, li.team);
            mvwaddstr(
                wnd,
                2 + li.position as i32,
                LEFT_BORDER_X_OFFSET,
                s.as_str(),
            );
            clrtoeol();
        }

        fmt::wreset(wnd);
        //RENDER SECOND WINDOW

        let wnd2 = self.track_wnd;

        fmt::wset_bold(wnd2);

        let title = "Relative Positions";
        let mut header = "Last ".to_string();
        header += (0..35).map(|_| "->").collect::<String>().as_str();
        header += " First";

        mvwaddstr(wnd2, 2, LEFT_BORDER_X_OFFSET, &title);

        mvwaddstr(wnd2, 3, LEFT_BORDER_X_OFFSET, &header);

        let mut positions_by_team: BTreeMap<Team, Vec<f32>> = BTreeMap::new();
        let mut max = -INFINITY;
        let mut min = INFINITY;
        for li in lap_info {
            if li.total_distance > max {
                max = li.total_distance;
            }
            if li.total_distance < min {
                min = li.total_distance
            }
            positions_by_team
                .entry(li.team)
                .or_insert_with(Vec::new)
                .push(li.total_distance)
        }
        let scale = max - min;
        let slice = scale / 80.0;
        let mut r = 3;
        for (team, positions) in positions_by_team {
            let mut row = (0..81).map(|_| " ").collect::<Vec<&str>>();
            for p in &positions {
                let place = ((*p - min) / slice) as usize;
                row[place] = "X"
            }
            let s = row.into_iter().collect::<String>();
            fmt::set_team_color(wnd2, team);
            mvwaddstr(wnd2, 1 + r, LEFT_BORDER_X_OFFSET, &s);
            r += 1
        }
        fmt::wreset(wnd2);

        self.refresh()
    }

    pub fn print_event_info(&self, event_info: &EventInfo) {
        fmt::set_bold();

        let mut msg = format!(
            "{}: {}",
            fmt::format_time_ms(event_info.timestamp),
            event_info.description
        );

        if let Some(driver) = event_info.driver_name {
            msg += &format!(": {}", driver);
        }

        if let Some(lap_time) = event_info.lap_time {
            msg += &format!(" ({})", fmt::format_time_ms(lap_time));
        }

        mvaddstr(getmaxy(self.mwnd) - 1, LEFT_BORDER_X_OFFSET, &msg);
        clrtoeol();

        fmt::reset();
    }

    pub fn print_telemetry_info(&self, telemetry_info: &TelemetryInfo) {
        let wnd = self.dashboard_wnd;

        fmt::set_bold();

        let gear_msg = format!(
            "Gear     : {}    Speed : {} KPH",
            fmt::format_gear(telemetry_info.gear),
            fmt::format_speed(telemetry_info.speed)
        );
        mvwaddstr(
            wnd,
            CURRENT_CAR_DATA_Y_OFFSET,
            LEFT_BORDER_X_OFFSET,
            &gear_msg,
        );

        mvwaddstr(
            wnd,
            CURRENT_CAR_DATA_Y_OFFSET + 1,
            LEFT_BORDER_X_OFFSET,
            "Throttle : ",
        );
        mvwaddstr(
            wnd,
            CURRENT_CAR_DATA_Y_OFFSET + 2,
            LEFT_BORDER_X_OFFSET,
            "Brake    : ",
        );

        let offset = getcurx(wnd);

        let throttle_bar = fmt::format_perc_bar(telemetry_info.throttle);
        fmt::wset_green(wnd);
        mvwaddstr(
            wnd,
            CURRENT_CAR_DATA_Y_OFFSET + 1,
            offset,
            &(throttle_bar + "                    "),
        );

        let brake_bar = fmt::format_perc_bar(telemetry_info.brake);
        fmt::wset_red(wnd);
        mvwaddstr(
            wnd,
            CURRENT_CAR_DATA_Y_OFFSET + 2,
            offset,
            &(brake_bar + "                    "),
        );

        fmt::wreset(wnd);
    }
}

fn addstr_center(w: WINDOW, y: i32, str_: &str) {
    mv(y, 0);
    clrtoeol();
    mvwaddstr(w, y, fmt::center(w, str_), str_);
}
