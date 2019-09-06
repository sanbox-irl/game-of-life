use super::{Color, Entity, MouseButton, Music, SoundPlayer, Sounds, SoundsVFX, State, Time, UserInput};
use anymap::AnyMap;
use std::{fmt::Debug, io::Cursor};
use winit::VirtualKeyCode as Key;

type UsizeTuple = (usize, usize);
type SoundFile = &'static [u8];

pub struct Gameplay {
    pub auto_increment: bool,
    pub coords_pressed: Vec<UsizeTuple>,
    pub show_debug: bool,
    pub show_instructions: bool,
    pub show_play_control: bool,
    pub show_settings_control: bool,
    pub increment_rate: f32,
    pub current_time: f32,
    pub playing: bool,
    pub show_ui: bool,
    pub game_colors: GameColors,
    pub game_sounds: GameSounds,
    pub saved_prefab: Option<Vec<Vec<State>>>,
    sound_player: SoundPlayer,
}

impl Gameplay {
    pub fn new(resources: &AnyMap) -> Self {
        Gameplay {
            auto_increment: false,
            coords_pressed: Vec::new(),
            increment_rate: 1.0,
            show_debug: true,
            current_time: 0.0,
            show_instructions: true,
            show_ui: true,
            playing: true,
            show_play_control: true,
            show_settings_control: false,
            game_colors: GameColors::default(),
            game_sounds: GameSounds::new(resources),
            sound_player: SoundPlayer::new(),
            saved_prefab: None,
        }
    }

    pub fn select(&mut self, click_pos: UsizeTuple, entities: &mut [Vec<Entity>]) {
        match &self.saved_prefab {
            Some(prefab) => {
                for this_x in click_pos.0..click_pos.0 + prefab.len() {
                    let command_x = this_x - click_pos.0;

                    for this_y in click_pos.1..click_pos.1 + prefab[command_x].len() {
                        let command_y = this_y - click_pos.1;

                        let entity = &mut entities[this_x][this_y];
                        entity.state = prefab[command_x][command_y];
                    }
                }

                self.saved_prefab = None;
                self.coords_pressed.push(click_pos);
                self.sound_player
                    .play_sound(Cursor::new(self.game_sounds.alive_sound));
            }

            None => {
                if self.coords_pressed.contains(&click_pos) == false {
                    let entity = &mut entities[click_pos.0][click_pos.1];
                    let new_state = entity.flip_state();
                    match new_state {
                        State::Alive => {
                            self.sound_player
                                .play_sound(Cursor::new(self.game_sounds.alive_sound));
                        }

                        State::Dead => {
                            self.sound_player
                                .play_sound(Cursor::new(self.game_sounds.dead_sound));
                        }

                        _ => {}
                    }
                    self.coords_pressed.push(click_pos);
                }
            }
        }
    }

    pub fn update(&mut self, user_input: &UserInput, entities: &mut [Vec<Entity>], time: &Time) {
        if user_input.mouse_input.is_released(MouseButton::Left) {
            self.coords_pressed.clear();
        }

        let mut do_not_update_again = false;
        if user_input.kb_input.is_pressed(Key::Return) {
            Gameplay::set_rules(entities);
            do_not_update_again = true;
            self.sound_player
                .play_sound(Cursor::new(self.game_sounds.tick_sound));
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

        if self.auto_increment && self.playing {
            self.current_time += time.delta_time;
            if self.increment_rate != 0.0 && self.current_time > (1.0 / self.increment_rate) {
                if do_not_update_again == false {
                    Gameplay::set_rules(entities);
                }
                self.current_time = 0.0;
            }
        }
    }

    pub fn set_rules(current_entities: &mut [Vec<Entity>]) {
        let mut ret: Vec<Vec<State>> = vec![];
        for (x, this_row) in current_entities.iter().enumerate() {
            let mut ret_row = vec![];
            for (y, entity) in this_row.iter().enumerate() {
                let current_pos = (x, y);
                let mut count = 0;

                // Check Up-Left
                if Self::entity_is_alive(current_entities, current_pos, Move::Negative, Move::Positive) {
                    count += 1;
                }

                // Check Up
                if Self::entity_is_alive(current_entities, current_pos, Move::Remain, Move::Positive) {
                    count += 1;
                }

                // Check Up-Right
                if Self::entity_is_alive(current_entities, current_pos, Move::Positive, Move::Positive) {
                    count += 1;
                }

                // Check Right
                if Self::entity_is_alive(current_entities, current_pos, Move::Positive, Move::Remain) {
                    count += 1;
                }

                // Check Down-Right
                if Self::entity_is_alive(current_entities, current_pos, Move::Positive, Move::Negative) {
                    count += 1;
                }

                // Check Down
                if Self::entity_is_alive(current_entities, current_pos, Move::Remain, Move::Negative) {
                    count += 1;
                }

                // Check Down-Left
                if Self::entity_is_alive(current_entities, current_pos, Move::Negative, Move::Negative) {
                    count += 1;
                }

                // Check Left
                if Self::entity_is_alive(current_entities, current_pos, Move::Negative, Move::Remain) {
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

    fn entity_is_alive(
        entities: &[Vec<Entity>],
        current_pos: (usize, usize),
        horizontal_move: Move,
        vertical_move: Move,
    ) -> bool {
        let entity = Self::get_entity(entities, current_pos, horizontal_move, vertical_move);
        entity.state == State::Alive
    }

    fn get_entity<T>(
        entities: &[Vec<T>],
        current_pos: (usize, usize),
        horizontal_move: Move,
        vertical_move: Move,
    ) -> &T {
        let x = Self::wrap(current_pos.0, horizontal_move, entities.len());
        let y = Self::wrap(current_pos.1, vertical_move.reverse(), entities[0].len());

        return &entities[x][y];
    }

    fn wrap(current: usize, move_amount: Move, wrap_size: usize) -> usize {
        if current == 0 && move_amount == Move::Negative {
            wrap_size - 1
        } else if current == wrap_size - 1 && move_amount == Move::Positive {
            0
        } else {
            match move_amount {
                Move::Positive => current + 1,
                Move::Negative => current - 1,
                Move::Remain => current,
            }
        }
    }
}

#[derive(PartialEq)]
enum Move {
    Positive,
    Negative,
    Remain,
}

impl Move {
    pub fn reverse(self) -> Self {
        match self {
            Move::Positive => Move::Negative,
            Move::Negative => Move::Positive,
            Move::Remain => Move::Remain,
        }
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
    alive_sound: SoundFile,
    dead_sound: SoundFile,
    tick_sound: SoundFile,
    music: SoundFile,
}

impl GameSounds {
    pub fn new(resources: &AnyMap) -> Self {
        let soundsfx: &SoundsVFX = resources.get().unwrap();

        let alive_sound = soundsfx.get_sound(Sounds::MakeCellAlive);
        let dead_sound = soundsfx.get_sound(Sounds::MakeCellDead);
        let tick_sound = soundsfx.get_sound(Sounds::Tick);

        let music = soundsfx.get_music(Music::Main);

        GameSounds {
            alive_sound,
            dead_sound,
            tick_sound,
            music,
        }
    }
}

impl<'a> Debug for GameSounds {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "Sounds")
    }
}
