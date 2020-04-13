use crate::compiler::semantic::{Rules, Condition};
use crate::compiler::parser::{NeighborCell, ComparisonOperator};
use rand::Rng;

pub struct Automaton {
    size: (usize, usize),
    grid: Vec<String>,
    grid_next: Vec<String>,
    rules: Rules
}

impl Automaton {
    pub fn new(rules: Rules) -> Automaton {
        let size = (200, 50);
        let initial_state = rules.initial_state.clone();
        let mut grid = vec![initial_state; size.0 * size.1];
        let mut rng = rand::thread_rng();
        for x in 0..size.0 {
            for y in 0..size.1 {
                let r: f32 = rng.gen();
                if r > 0.5 {
                    let index = y * size.0 + x;
                    grid[index] = "dead".to_string();
                }
            }
        }
        let grid_next = grid.clone();

        Automaton {
            size,
            grid,
            grid_next,
            rules
        }
    }

    pub fn tick(&mut self) {
        for x in 0..self.size.0 {
            for y in 0..self.size.1 {
                let index = self.get_index(x as isize, y as isize);
                let state_name = self.grid[index].clone();
                for (state_origin, state_destination, conditions) in &self.rules.transitions {
                    if state_origin == &state_name {
                        let mut conditions_evaluation = false;
                        for conjunction in conditions {
                            let mut conjunction_evaluation = true;
                            for condition in conjunction {
                                match condition {
                                    Condition::QuantityCondition(state, comp, quantity) => {
                                        let mut count = 0;
                                        for u in -1..2 {
                                            for v in -1..2 {
                                                if u != 0 || v != 0 {
                                                    let index_2 =  self.get_index(x as isize + u, y as isize + v);
                                                    if &self.grid[index_2] == state {
                                                        count += 1;
                                                    }
                                                }
                                            }
                                        }
                                        let condition_evaluation = match comp {
                                            ComparisonOperator::Greater => &count > quantity,
                                            ComparisonOperator::Lesser => &count < quantity,
                                            ComparisonOperator::GreaterOrEqual => &count >= quantity,
                                            ComparisonOperator::LesserOrEqual => &count <= quantity,
                                            ComparisonOperator::Equal => &count == quantity,
                                            ComparisonOperator::Different => &count != quantity
                                        };
                                        if !condition_evaluation {
                                            conjunction_evaluation = false;
                                            break;
                                        }
                                    },
                                    Condition::NeighborCondition(neighbor, state) => {
                                        let index = self.get_index_of_neighbor(x as isize, y as isize, neighbor);
                                        if &self.grid[index] != state {
                                            conjunction_evaluation = false;
                                            break;
                                        }
                                    },
                                    Condition::True => {
                                        break;
                                    }
                                }
                            }
                            if conjunction_evaluation {
                                conditions_evaluation = true;
                                break;
                            }
                        }

                        if conditions_evaluation {
                            self.grid_next[index] = state_destination.clone();
                        }
                    }
                }
            }
        }

        for x in 0..self.size.0 {
            for y in 0..self.size.1 {
                let index = self.get_index(x as isize, y as isize);
                self.grid[index] = self.grid_next[index].clone();
            }
        }
    }

    fn get_index_of_neighbor(& self, x: isize, y: isize, neighbor: &NeighborCell) -> usize {
        let (x_n, y_n) = match neighbor {
            NeighborCell::A => (x - 1, y - 1),
            NeighborCell::B => (x, y - 1),
            NeighborCell::C => (x + 1, y - 1),
            NeighborCell::D => (x - 1, y),
            NeighborCell::E => (x + 1, y),
            NeighborCell::F => (x - 1, y + 1),
            NeighborCell::G => (x, y + 1),
            NeighborCell::H => (x + 1, y + 1)
        };
        self.get_index(x_n, y_n)
    }

    fn get_index(&self, x: isize, y: isize) -> usize {
        Self::tore_correction(y, self.size.1) * self.size.0 + Self::tore_correction(x, self.size.0)
    }

    /// The world is a tore, so the value range can be )-inf; +inf(, and it will be mapped to (0; upper_bound-1).
    fn tore_correction(value: isize, upper_bound: usize) -> usize {
        if value >= 0 {
            (value as usize) % upper_bound
        }
        else {
            let signed_upper_bound = upper_bound as isize;
            let corrected = signed_upper_bound + (value % signed_upper_bound);
            corrected as usize
        }
    }

    pub fn get_color(&self, x: isize, y: isize) -> (u8, u8, u8) {
        let state_name = &self.grid[self.get_index(x, y)];
        self.rules.states.get(state_name).unwrap().clone()
    }
}
