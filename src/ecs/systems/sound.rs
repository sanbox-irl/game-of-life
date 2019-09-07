use rodio::{Decoder, Device, Sink};
use std::io::Cursor;

pub struct SoundPlayer {
    device: Device,
}

impl SoundPlayer {
    pub fn new() -> Self {
        let device: rodio::Device = rodio::default_output_device().unwrap();
        SoundPlayer {
            device,
        }
    }

    pub fn play_sound(&mut self, sound: Cursor<&'static [u8]>, volume: f32) {
        let decoder = Decoder::new(sound).unwrap();
        let sink = Sink::new(&self.device);
        sink.set_volume(volume);
        sink.append(decoder);
        sink.detach();
    }
}
