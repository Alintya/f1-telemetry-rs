use super::fmt;
use crate::models::CarStatus;
use ncurses::*;

const LEFT_FRONT_WING: &str = "\
___________/--
||||||||||||||
|_________/--|
|            |";

const RIGHT_FRONT_WING: &str = "\
--\\___________
||||||||||||||
|--\\_________|
|            |";

const TYRE: &str = " _____
|     |
|     |
|     |
|_____|";

const BODY: &str = "      /  \\
     /    \\
----/      \\----
---/        \\---
  /          \\
 /            \\
 |            |
 |            |
 |            |
 \\            /
  \\          /
---|        |---
---|        |---
   |        |   ";

const ENGINE: &str = "__--__
|    |
\\    /
 \\__/";

const GEARBOX: &str = "||
||";

const REAR_WING: &str = "\
|------------------|
|------------------|
|------------------|";

pub fn render_car(w: WINDOW, car_status: &CarStatus) {
    render_component(w, LEFT_FRONT_WING, car_status.left_front_wing_damage, 0, 1);
    render_component(
        w,
        RIGHT_FRONT_WING,
        car_status.right_front_wing_damage,
        0,
        15,
    );
    render_component(w, BODY, 0, 4, 7);
    render_component(w, ENGINE, car_status.engine_damage, 11, 12);
    render_component(w, GEARBOX, car_status.gearbox_damage, 15, 14);
    render_component(w, REAR_WING, car_status.rear_wing_damage, 18, 5);

    render_component(w, TYRE, car_status.tyres_damage.front_left(), 4, 0);
    render_component(w, TYRE, car_status.tyres_damage.front_right(), 4, 22);
    render_component(w, TYRE, car_status.tyres_damage.rear_left(), 13, 0);
    render_component(w, TYRE, car_status.tyres_damage.rear_right(), 13, 22);
}

fn render_component(w: WINDOW, component: &str, damage: u8, y: i32, x: i32) {
    fmt::set_damage_color(Some(w), damage);

    for (i, l) in component.split('\n').enumerate() {
        mvwaddstr(w, y + i as i32, x, l);
    }

    fmt::wreset(w);
}
