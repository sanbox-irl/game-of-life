#![warn(elided_lifetimes_in_paths)]

extern crate env_logger;
extern crate winit;
#[macro_use]
extern crate log;
#[macro_use]
extern crate memoffset;
extern crate gfx_hal;
extern crate imgui;

mod ecs;
mod game;
mod rendering;
mod utilities;

fn main() {
    env_logger::init();

    let mut game = match game::Game::new() {
        Ok(game) => game,
        Err(err) => {
            error!("{}", err);
            return;
        }
    };
    let clean_exit = game.main_loop();

    if clean_exit {
        info!("Exiting cleanly and gracefully.");
    } else {
        error!("Exiting with error.");
    }
}
