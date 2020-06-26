use crate::models::{EventInfo, LapInfo, SessionInfo};
use f1_telemetry::packet::lap::ResultStatus;
use ncurses::*;

mod fmt;

const MIN_WIDTH: i32 = 84;
const SESSION_Y_OFFSET: i32 = 1;
const WINDOW_Y_OFFSET: i32 = 5;

pub enum Window {
    Lap,
    Car,
}

pub struct Ui {
    mwnd: WINDOW,
    lap_wnd: WINDOW,
    car_wnd: WINDOW,
    active_wnd: WINDOW,
}

impl Ui {
    pub fn init() -> Ui {
        setlocale(ncurses::LcCategory::all, "");

        let mwnd = initscr();

        let w = getmaxx(mwnd);
        let h = getmaxy(mwnd);

        if w < MIN_WIDTH {
            panic!("Terminal too narrow");
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

        let lap_wnd = Ui::create_win(win_h, win_w, WINDOW_Y_OFFSET, 1, Some("Lap Info"));
        let car_wnd = Ui::create_win(win_h, win_w, WINDOW_Y_OFFSET, 1, Some("Car Status"));

        let active_wnd = lap_wnd;
        wrefresh(active_wnd);

        Ui {
            mwnd,
            lap_wnd,
            car_wnd,
            active_wnd,
        }
    }

    pub fn destroy(&self) {
        endwin();
    }

    pub fn switch_window(&mut self, window: Window) {
        let neww = match window {
            Window::Lap => self.lap_wnd,
            Window::Car => self.car_wnd,
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
        let wnd = self.lap_wnd;

        fmt::wset_bold(wnd);

        let header =
            "  P. NAME                 | CURRENT LAP  | LAST LAP     | BEST LAP     | STATUS";
        let x_offset = (getmaxx(wnd) - header.len() as i32) / 2;

        mvwaddstr(wnd, 1, x_offset, header);

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
            mvwaddstr(wnd, 2 + li.position as i32, x_offset, s.as_str());
            clrtoeol();
        }

        fmt::wreset(wnd);

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

        mvaddstr(getmaxy(self.mwnd) - 1, 2, &msg);
        clrtoeol();

        fmt::reset();
    }
}

fn addstr_center(w: WINDOW, y: i32, str_: &str) {
    mv(y, 0);
    clrtoeol();
    mvwaddstr(w, y, fmt::center(w, str_), str_);
}
