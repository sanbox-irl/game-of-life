use super::{Entity, MouseButton, State, Time, UserInput};
use winit::VirtualKeyCode as Key;

type UsizeTuple = (usize, usize);

#[derive(Debug)]
pub struct Gameplay {
    pub auto_increment: bool,
    pub increment_rate: f32,
    pub current_time: f32,
    pub playing: bool,
    pub coords_pressed: Vec<UsizeTuple>,
    pub show_debug: bool,
    pub show_instructions: bool,
    pub show_ui: bool,
}

impl Gameplay {
    pub fn select(&mut self, coord_pos: UsizeTuple, entities: &mut [Vec<Entity>]) {
        if self.coords_pressed.contains(&coord_pos) == false {
            let entity = &mut entities[coord_pos.0][coord_pos.1];
            entity.flip_state();
            self.coords_pressed.push(coord_pos);
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

        self.current_time += time.delta_time;
        if self.increment_rate != 0.0 && self.current_time > self.increment_rate {
            if do_not_update_again == false {
                Gameplay::set_rules(entities);
            }
            self.current_time = 0.0;
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

impl Default for Gameplay {
    fn default() -> Self {
        Gameplay {
            auto_increment: false,
            coords_pressed: Vec::new(),
            increment_rate: 0.0,
            show_debug: true,
            current_time: 0.0,
            show_instructions: true,
            show_ui: true,
            playing: false,
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
