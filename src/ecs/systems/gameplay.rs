use super::{
    simple_serialization, Color, Entity, MouseButton, Music, Prefab, SoundPlayer, Sounds, SoundsVFX, State,
    Time, UserInput, Vec2, Vec2Int,
};
use anymap::AnyMap;
use rodio::Sink;
use std::{fmt::Debug, io::Cursor};
use winit::VirtualKeyCode as Key;

type UsizeTuple = (usize, usize);
type SoundFile = &'static [u8];

pub struct Gameplay {
    pub show_debug: bool,
    pub auto_increment: bool,
    pub show_instructions: bool,
    pub show_play_control: bool,
    pub show_settings_control: bool,
    pub increment_rate: f32,
    pub current_time: f32,
    pub playing: bool,
    pub show_ui: bool,
    pub game_colors: GameColors,
    pub game_sounds: GameSounds,
    pub saved_prefab: Option<Prefab>,
    pub wrap_grid: bool,
    game_size: Vec2,
    next_game_size: Option<Vec2>,
    coords_pressed: Vec<UsizeTuple>,
    prefabs: Prefabs,
    sound_player: SoundPlayer,
    flags: GameplayFlags,
}

impl Gameplay {
    pub fn new(resources: &AnyMap, game_size: Vec2) -> Result<Self, Error> {
        let sound_player = SoundPlayer::new();
        let music_sink = sound_player.make_sink();

        let this = Gameplay {
            auto_increment: false,
            coords_pressed: Vec::new(),
            increment_rate: 1.0,
            show_debug: true,
            current_time: 0.0,
            show_instructions: true,
            show_ui: true,
            playing: true,
            wrap_grid: false,
            show_play_control: true,
            show_settings_control: false,
            game_colors: GameColors::default(),
            game_sounds: GameSounds::new(resources, music_sink),
            sound_player: SoundPlayer::new(),
            saved_prefab: None,
            prefabs: Prefabs::new()?,
            game_size,
            next_game_size: None,
            flags: GameplayFlags::empty(),
        };

        Ok(this)
    }

    pub fn game_size(&self) -> Vec2Int {
        self.game_size.into()
    }

    pub fn next_game_size(&self) -> Vec2Int {
        match self.next_game_size {
            Some(gz) => gz.into(),
            None => self.game_size.into(),
        }
    }

    pub fn set_next_game_size(&mut self, game_size: Vec2Int) {
        self.next_game_size = Some(game_size.into());
    }

    pub fn resize_this_frame(&mut self) {
        self.flags.insert(GameplayFlags::RESIZE);
    }

    pub fn select(&mut self, click_pos: UsizeTuple, entities: &mut [Vec<Entity>]) {
        match &self.saved_prefab {
            Some(prefab) => {
                if let Some(prefab) = self.prefabs.prefabs.get(&prefab) {
                    Self::paste_cells(click_pos, prefab, entities);

                    self.saved_prefab = None;
                    self.coords_pressed.push(click_pos);
                    self.sound_player.play_sound(
                        Cursor::new(self.game_sounds.alive_sound),
                        self.game_sounds.sfx_volume,
                    );

                    return;
                } else {
                    println!("Couldn't find prefab by name of {:?}", prefab);
                }
            }

            None => {}
        }

        if self.coords_pressed.contains(&click_pos) == false {
            let entity = &mut entities[click_pos.0][click_pos.1];
            let new_state = entity.flip_state();
            match new_state {
                State::Alive => {
                    self.sound_player.play_sound(
                        Cursor::new(self.game_sounds.alive_sound),
                        self.game_sounds.sfx_volume,
                    );
                }

                State::Dead => {
                    self.sound_player.play_sound(
                        Cursor::new(self.game_sounds.dead_sound),
                        self.game_sounds.sfx_volume,
                    );
                }

                _ => {}
            }
            self.coords_pressed.push(click_pos);
        }
    }

    pub fn new_size(&mut self, entities: &mut [Vec<Entity>]) -> Option<Vec<Vec<Entity>>> {
        if self.flags.contains(GameplayFlags::RESIZE) {
            if let Some(next_size) = self.next_game_size {
                let mut new_entities = Self::create_game_world(next_size);

                let old_world = Self::to_pure_states(entities);
                let center_offset: Vec2 = next_size / 2.0 - self.game_size / 2.0;
                let center_round_down: Vec2Int = center_offset.into();

                // Throw away if under zero.
                // in the future, we can sheer
                if !(center_round_down.x < 0 || center_round_down.y < 0) {
                    Self::paste_cells(
                        center_round_down.into_raw_usize().unwrap(),
                        &old_world,
                        &mut new_entities,
                    );
                }
                self.game_size = next_size;
                self.flags.remove(GameplayFlags::RESIZE);
                Some(new_entities)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn update(&mut self, user_input: &UserInput, entities: &mut [Vec<Entity>], time: &Time) {
        if user_input.mouse_input.is_released(MouseButton::Left) {
            self.coords_pressed.clear();
        }

        let mut do_not_update_again = false;
        if user_input.kb_input.is_pressed(Key::Return) {
            self.set_rules(entities);
            do_not_update_again = true;
            self.sound_player.play_sound(
                Cursor::new(self.game_sounds.tick_sound),
                self.game_sounds.sfx_volume,
            );
        }

        if user_input.kb_input.is_pressed(Key::Space) {
            self.playing = !self.playing;
        }

        if user_input.kb_input.is_pressed(Key::F1) {
            self.show_ui = !self.show_ui;
        }

        if user_input.kb_input.is_pressed(Key::F2) {
            self.show_instructions = !self.show_instructions;
        }

        if user_input.kb_input.is_pressed(Key::F10) {
            self.show_debug = !self.show_debug;
        }

        if user_input.kb_input.is_pressed(Key::F3) {
            let states = Self::to_pure_states(entities);
            simple_serialization::save(&states, "okay.json").unwrap();
        }

        if self.auto_increment && self.playing {
            self.current_time += time.delta_time;
            if self.increment_rate != 0.0 && self.current_time > (1.0 / self.increment_rate) {
                if do_not_update_again == false {
                    self.set_rules(entities);
                }
                self.current_time = 0.0;
            }
        }
    }

    pub fn set_rules(&self, current_entities: &mut [Vec<Entity>]) {
        let mut ret: Vec<Vec<State>> = vec![];
        for (x, this_row) in current_entities.iter().enumerate() {
            let mut ret_row = vec![];
            for (y, entity) in this_row.iter().enumerate() {
                let current_pos = Vec2Int::new(x as i32, y as i32);
                let mut count = 0;

                // Check Up-Left
                if self.entity_is_alive(current_entities, current_pos - Vec2Int::RIGHT + Vec2Int::UP) {
                    count += 1;
                }

                // Check Up
                if self.entity_is_alive(current_entities, current_pos + Vec2Int::UP) {
                    count += 1;
                }

                // Check Up-Right
                if self.entity_is_alive(current_entities, current_pos + Vec2Int::RIGHT + Vec2Int::UP) {
                    count += 1;
                }

                // Check Right
                if self.entity_is_alive(current_entities, current_pos + Vec2Int::RIGHT) {
                    count += 1;
                }

                // Check Down-Right
                if self.entity_is_alive(current_entities, current_pos + Vec2Int::RIGHT - Vec2Int::UP) {
                    count += 1;
                }

                // Check Down
                if self.entity_is_alive(current_entities, current_pos - Vec2Int::UP) {
                    count += 1;
                }

                // Check Down-Left
                if self.entity_is_alive(current_entities, current_pos - Vec2Int::RIGHT - Vec2Int::UP) {
                    count += 1;
                }

                // Check Left
                if self.entity_is_alive(current_entities, current_pos - Vec2Int::RIGHT) {
                    count += 1;
                }

                if entity.state == State::Alive {
                    ret_row.push(match count {
                        2..=3 => State::Alive,
                        _ => State::Dead,
                    });
                } else {
                    ret_row.push(if count == 3 {
                        State::Alive
                    } else {
                        if entity.state == State::Dead {
                            State::Dead
                        } else {
                            State::Unborn
                        }
                    });
                }
            }
            ret.push(ret_row);
        }

        for (x, this_row) in current_entities.iter_mut().enumerate() {
            for (y, entity) in this_row.iter_mut().enumerate() {
                entity.state = ret[x][y];
            }
        }
    }

    fn entity_is_alive(&self, entities: &[Vec<Entity>], pos: Vec2Int) -> bool {
        if self.wrap_grid {
            let x = Self::wrap(pos.x, entities.len());
            let y = Self::wrap(pos.y, entities[0].len());
            entities[x][y].state == State::Alive
        } else {
            if pos.x > 0 && pos.x < entities.len() as i32 && pos.y > 0 && pos.y < entities[0].len() as i32 {
                entities[pos.x as usize][pos.y as usize].state == State::Alive
            } else {
                false
            }
        }
    }

    fn wrap(current: i32, wrap_size: usize) -> usize {
        if current < 0 {
            wrap_size - 1
        } else if current as usize == wrap_size {
            0
        } else {
            current as usize
        }
    }

    fn paste_cells(click_pos: UsizeTuple, prefab: &[Vec<State>], entities: &mut [Vec<Entity>]) {
        for this_x in click_pos.0..click_pos.0 + prefab.len() {
            let command_x = this_x - click_pos.0;

            for this_y in click_pos.1..click_pos.1 + prefab[command_x].len() {
                if this_x >= entities.len() || this_y >= entities[this_x].len() {
                    continue;
                }
                let command_y = this_y - click_pos.1;

                let entity = &mut entities[this_x][this_y];
                let new_state = prefab[command_x][command_y];
                if !(entity.state == State::Unborn && new_state == State::Dead) {
                    entity.state = new_state;
                }
            }
        }
    }

    pub fn create_game_world(size: Vec2) -> Vec<Vec<Entity>> {
        let mut entities = vec![];
        for x in 0..size.x as i32 {
            let mut this_vec = vec![];
            for y in 0..size.y as i32 {
                this_vec.push(Entity::new(Vec2::new(x as f32, y as f32)));
            }
            entities.push(this_vec);
        }
        entities
    }

    fn to_pure_states(entities: &mut [Vec<Entity>]) -> Vec<Vec<State>> {
        let mut ret = vec![];
        for this_row in entities.iter() {
            let mut ret_row = vec![];
            for entity in this_row {
                ret_row.push(entity.state)
            }
            ret.push(ret_row);
        }
        ret
    }
}

#[derive(Debug)]
pub struct GameColors {
    pub alive: Color,
    pub dead: Color,
    pub unborn: Color,
    pub bg: Color,
    pub grid_lines: bool,
    pub grid_line_width: f32,
    pub grid_line_color: Color,
}

impl GameColors {
    pub fn get_color(&self, state: State) -> &Color {
        match state {
            State::Alive => &self.alive,
            State::Dead => &self.dead,
            State::Unborn => &self.unborn,
        }
    }
}

impl Default for GameColors {
    fn default() -> Self {
        Self {
            alive: Color::with_u8(17, 54, 12),
            dead: Color::with_u8(47, 29, 24),
            unborn: Color::with_u8(139, 110, 101),
            bg: Color::with_u8(139, 110, 101),
            grid_lines: true,
            grid_line_width: 0.025,
            grid_line_color: Color::new(0.2, 0.5, 0.1),
        }
    }
}

pub struct GameSounds {
    pub sfx_volume: f32,
    msc_volume: f32,
    alive_sound: SoundFile,
    dead_sound: SoundFile,
    tick_sound: SoundFile,
    music_sink: Sink,
}

impl GameSounds {
    pub fn new(resources: &AnyMap, mut music_sink: Sink) -> Self {
        let sounds_sfx: &SoundsVFX = resources.get().unwrap();

        let alive_sound = sounds_sfx.get_sound(Sounds::MakeCellAlive);
        let dead_sound = sounds_sfx.get_sound(Sounds::MakeCellDead);
        let tick_sound = sounds_sfx.get_sound(Sounds::Tick);

        let intro_music = sounds_sfx.get_music(Music::Intro);
        let music = sounds_sfx.get_music(Music::Main);
        SoundPlayer::load_sink(&mut music_sink, intro_music);
        SoundPlayer::load_sink_infinite(&mut music_sink, music);

        GameSounds {
            alive_sound,
            dead_sound,
            tick_sound,
            sfx_volume: 1.0,
            msc_volume: 1.0,
            music_sink,
        }
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.msc_volume = volume;
        self.music_sink.set_volume(self.msc_volume);
    }

    pub fn music_volume(&self) -> f32 {
        self.msc_volume
    }
}

impl<'a> Debug for GameSounds {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "Sounds")
    }
}

use failure::Error;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Prefabs {
    prefabs: HashMap<Prefab, Vec<Vec<State>>>,
}

impl Prefabs {
    pub fn new() -> Result<Prefabs, Error> {
        let mut prefabs: HashMap<Prefab, Vec<Vec<State>>> = HashMap::new();

        prefabs.insert(
            Prefab::Glider,
            simple_serialization::load("resources/prefabs/glider.json")?,
        );
        prefabs.insert(
            Prefab::SmallExploder,
            simple_serialization::load("resources/prefabs/small_exploder.json")?,
        );
        prefabs.insert(
            Prefab::Exploder,
            simple_serialization::load("resources/prefabs/exploder.json")?,
        );
        prefabs.insert(
            Prefab::Spaceship,
            simple_serialization::load("resources/prefabs/lw_spaceship.json")?,
        );
        prefabs.insert(
            Prefab::Tumbler,
            simple_serialization::load("resources/prefabs/tumbler.json")?,
        );
        prefabs.insert(
            Prefab::GliderGun,
            simple_serialization::load("resources/prefabs/glider_gun.json")?,
        );

        Ok(Prefabs { prefabs })
    }
}

bitflags! {
    struct GameplayFlags: u32 {
        const RESIZE = 0b00000001;
    }
}
