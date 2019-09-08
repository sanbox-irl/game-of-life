use rodio::{Decoder, Device, Sink, Source};
use std::io::Cursor;

pub struct SoundPlayer {
    device: Device,
}

impl SoundPlayer {
    pub fn new() -> Self {
        let device: rodio::Device = rodio::default_output_device().unwrap();
        SoundPlayer { device }
    }

    pub fn play_sound(&self, sound: Cursor<&'static [u8]>, volume: f32) {
        let decoder = Decoder::new(sound).unwrap();
        let sink = Sink::new(&self.device);
        sink.set_volume(volume);
        sink.append(decoder);
        sink.detach();
    }

    pub fn make_sink(&self) -> Sink {
        Sink::new(&self.device)
    }

    pub fn load_sink(sink: &mut Sink, music: &'static [u8]) {
        let decoder = Decoder::new(Cursor::new(music)).unwrap();
        sink.append(decoder);
    }

    pub fn load_sink_infinite(sink: &mut Sink, music: &'static [u8]) {
        let decoder = Decoder::new(Cursor::new(music)).unwrap().repeat_infinite();
        sink.append(decoder);
    }
}
