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

macro_rules! get_sound {
    ($name:expr) => {
        &include_bytes!($name)[..]
    };
}

impl SoundsVFX {
    pub fn new() -> Self {
        let alive = get_sound!("../../resources/sounds/make_alive.wav");
        let dead = get_sound!("../../resources/sounds/make_dead.wav");
        let tick = get_sound!("../../resources/sounds/tick.wav");
        let music_intro = get_sound!("../../resources/music/game_of_life_intro.ogg");
        let music_main = get_sound!("../../resources/music/game_of_life_main.ogg");

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
