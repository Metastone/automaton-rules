use crate::compiler::semantic::{Rules, Condition};
use crate::compiler::parser::{NeighborCell, ComparisonOperator};
use rand::Rng;

type StateId = usize;

pub struct Automaton {
    size: (usize, usize),
    grid: Vec<String>,
    grid_next: Vec<String>,
    rules: Rules
}

impl Automaton {
    pub fn new(rules: Rules) -> Automaton {
        let size = (200, 100);
        let initial_state = rules.initial_state.clone();
        let mut grid = vec![initial_state; (size.0+2)*(size.1+2)];
        for x in 0..(size.0+2) {
            grid[x] = "".to_string();
            grid[(size.1+1) * (size.0+2) + x] = "".to_string();
        }
        for y in 0..(size.1+2) {
            grid[y * (size.0+2)] = "".to_string();
            grid[y * (size.0+2) + size.0 + 1] = "".to_string();
        }
        let mut rng = rand::thread_rng();
        for x in 1..(size.0+1) {
            for y in 1..(size.1+1) {
                let r: f32 = rng.gen();
                if r > 0.5 {
                    let index = y*(size.0+2) + x;
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
        for x in 1..(self.size.0+1) {
            for y in 1..(self.size.1+1) {
                let index = y*(self.size.0+2) + x;
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
                                                    let index_2 = ((y as i32 + v) as usize)*(self.size.0+2) + ((x as i32 + u) as usize);
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
                                        let index = match neighbor {
                                            NeighborCell::A => (y-1) * (self.size.0+2) + x-1,
                                            NeighborCell::B => (y-1) * (self.size.0+2) + x,
                                            NeighborCell::C => (y-1) * (self.size.0+2) + x+1,
                                            NeighborCell::D => y * (self.size.0+2) + x-1,
                                            NeighborCell::E => y * (self.size.0+2) + x+1,
                                            NeighborCell::F => (y+1) * (self.size.0+2) + x-1,
                                            NeighborCell::G => (y+1) * (self.size.0+2) + x,
                                            NeighborCell::H => (y+1) * (self.size.0+2) + x+1
                                        };
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

        for x in 1..(self.size.0+1) {
            for y in 1..(self.size.1+1) {
                let index = y*(self.size.0+2) + x;
                self.grid[index] = self.grid_next[index].clone();
            }
        }
    }

    pub fn get_color(&self, state_name: &String) -> (u8, u8, u8) {
        self.rules.states.get(state_name).unwrap().clone()
    }

    pub fn get_size(&self) -> &(usize, usize) {
        &self.size
    }

    pub fn get_grid(&self) -> &Vec<String> {
        &self.grid
    }
}
