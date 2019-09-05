#![warn(elided_lifetimes_in_paths)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate memoffset;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate imgui;

use rodio::Sink;

mod ecs;
mod game;
mod rendering;
mod utilities;

fn main() {
    env_logger::init();

    /*
    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);=
    let source = rodio::source::SineWave::new(440);
    sink.append(source);
    sink.set_volume(0.2);
    */

    let mut game = match game::Game::new() {
        Ok(game) => game,
        Err(e) => {
            error!("{}", e);
            let causes = e.iter_causes();
            for this_cause in causes {
                error!("{}", this_cause);
            }
            return;
        }
    };
    let end_game = game.main_loop();
    match end_game {
        Ok(()) => {
            info!("Exiting cleanly and gracefully.");
        }

        Err(e) => {
            error!("{}", e);
            error!("{}", e.backtrace())
        }
    };
}
