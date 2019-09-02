use super::{Entity, MouseButton, State, UserInput};

#[derive(Debug, Default)]
pub struct Gameplay {
    pub auto_increment: bool,
    pub increment_rate: f32,
    pub single_selection: bool,
    coords_pressed: Vec<(usize, usize)>,
}

impl Gameplay {
    pub fn select(&mut self, coord_pos: (usize, usize), entity: &mut Entity) {
        if self.single_selection && self.coords_pressed.contains(&coord_pos) == false {
            entity.flip_state();
            self.coords_pressed.push(coord_pos);
        }
    }

    pub fn update_inputs(&mut self, user_input: &UserInput) {
        if user_input.mouse_input.is_released(MouseButton::Left) {
            self.coords_pressed.clear();
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
