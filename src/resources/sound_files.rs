use rodio::{Sink, Source};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug)]
pub enum Sounds {
    MakeCellAlive = 0,
    MakeCellDead,
    Tick,
}

#[derive(Debug)]
pub enum Music {
    Main,
}

pub struct SoundsVFX {
    sounds: Vec<&'static [u8]>,
    music: Vec<&'static [u8]>,
}

impl SoundsVFX {
    pub fn new(&self) -> Self {
        let file = File::open("test.ogg").unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap().buffered();
        let res = source.convert_samples();

        let device = rodio::default_output_device().unwrap();
        let sink = Sink::new(&device);
        sink.append(source);
    }
}
