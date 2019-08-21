use super::State;

#[allow(dead_code)]
pub fn set_rules(current_entities: &[Vec<State>]) -> Vec<Vec<State>> {
    let mut ret = vec![];
    for (y, this_row) in current_entities.iter().enumerate() {
        let mut ret_row = vec![];
        for (x, entity) in this_row.iter().enumerate() {
            let current_pos = (x, y);
            let mut count = 0;

            // Check Up-Left
            if entity_is_alive(current_entities, current_pos, Move::Negative, Move::Positive) {
                count += 1;
            }

            // Check Up
            if entity_is_alive(current_entities, current_pos, Move::Remain, Move::Positive) {
                count += 1;
            }

            // Check Up-Right
            if entity_is_alive(current_entities, current_pos, Move::Positive, Move::Positive) {
                count += 1;
            }

            // Check Right
            if entity_is_alive(current_entities, current_pos, Move::Positive, Move::Remain) {
                count += 1;
            }

            // Check Down-Right
            if entity_is_alive(current_entities, current_pos, Move::Positive, Move::Negative) {
                count += 1;
            }

            // Check Down
            if entity_is_alive(current_entities, current_pos, Move::Remain, Move::Negative) {
                count += 1;
            }

            // Check Down-Left
            if entity_is_alive(current_entities, current_pos, Move::Negative, Move::Negative) {
                count += 1;
            }

            // Check Left
            if entity_is_alive(current_entities, current_pos, Move::Negative, Move::Remain) {
                count += 1;
            }

            if entity == &State::Alive {
                ret_row.push(match count {
                    2..=3 => State::Alive,
                    _ => State::Dead,
                });
            } else {
                ret_row.push(if count == 3 { State::Alive } else { State::Dead });
            }
        }
        ret.push(ret_row);
    }
    ret
}

fn entity_is_alive(
    entities: &[Vec<State>],
    current_pos: (usize, usize),
    horizontal_move: Move,
    vertical_move: Move,
) -> bool {
    let entity = get_entity(entities, current_pos, horizontal_move, vertical_move);
    entity == &State::Alive
}

fn get_entity(
    entities: &[Vec<State>],
    current_pos: (usize, usize),
    horizontal_move: Move,
    vertical_move: Move,
) -> &State {
    let x = wrap(current_pos.0, horizontal_move, entities[0].len());
    let y = wrap(current_pos.1, vertical_move.reverse(), entities.len());

    return &entities[y][x];
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
