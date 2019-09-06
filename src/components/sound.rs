use rodio::{Decoder, Device, Sink, Source};
use std::io::Cursor;
use std::sync::Arc;

pub struct SoundPlayer {
    device: Device,
    sinks: Vec<Sink>,
}

impl SoundPlayer {
    pub fn new() -> Self {
        let device: rodio::Device = rodio::default_output_device().unwrap();
        SoundPlayer {
            sinks: vec![Sink::new(&device)],
            device,
        }
    }

    pub fn play_sound(&mut self, sound: Cursor<&'static [u8]>) {
        let decoder = Decoder::new(sound).unwrap();
        let sink = Sink::new(&self.device);
        sink.append(decoder);
        sink.detach();
    }

    fn get_sink(&mut self) -> Option<&Sink> {
        for this_sink in self.sinks.iter() {
            if this_sink.empty() {
                return Some(this_sink);
            }
        }

        None

        // let this_sink = Sink::new(&self.device);
        // self.sinks.push(this_sink);
        // &this_sink
    }
}
