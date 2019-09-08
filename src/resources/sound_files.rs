use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Sounds {
    MakeCellAlive,
    MakeCellDead,
    Tick,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Music {
    Intro,
    Main,
}

pub struct SoundsVFX {
    sounds: HashMap<Sounds, &'static [u8]>,
    music: HashMap<Music, &'static [u8]>,
}


impl SoundsVFX {
    pub fn new() -> Self {
        let alive = &include_bytes!("../../resources/sounds/create.ogg")[..];
        let dead = &include_bytes!("../../resources/sounds/kill.ogg")[..];
        let tick = &include_bytes!("../../resources/sounds/tick.ogg")[..];
        let music_intro = &include_bytes!("../../resources/music/intro.ogg")[..];
        let music_main = &include_bytes!("../../resources/music/main.ogg")[..];

        SoundsVFX {
            sounds: hashmap![
                Sounds::MakeCellAlive => alive,
                Sounds::MakeCellDead => dead,
                Sounds::Tick => tick
            ],
            music: hashmap![
                Music::Intro => music_intro,
                Music::Main => music_main,
            ],
        }
    }

    pub fn get_sound(&self, sound: Sounds) -> &'static [u8] {
        self.sounds.get(&sound).unwrap()
    }

    pub fn get_music(&self, music: Music) -> &'static [u8] {
        self.music.get(&music).unwrap()
    }
}
