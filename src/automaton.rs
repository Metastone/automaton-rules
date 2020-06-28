use crate::compiler::semantic::{State, Rules, Condition, StateDistribution};
use crate::compiler::parser::{NeighborCell, ComparisonOperator};
use rand::{Rng, rngs::ThreadRng};
use rayon::prelude::*;

#[derive(Clone)]
pub struct Cell {
    state: usize,
    index_in_grid: usize,
}

pub struct Automaton {
    grid: Vec<Cell>,
    grid_next: Vec<Cell>,
    rules: Rules,
}

impl Automaton {
    pub fn new(rules: Rules) -> Automaton {
        let size = &rules.world_size;
        let states = &rules.states;

        // Initialize grid with default state.
        let default_state = rules.states.iter()
            .find(|s| match s.distribution {
                StateDistribution::Default => true,
                _ => false
            })
            .unwrap().id;
        let mut grid = Vec::new();
        for i in 0..(size.0 * size.1) {
            grid.push(Cell{
                state: default_state,
                index_in_grid: i
            });
        }

        // Add the states that have a proportion distribution.
        Self::add_p_distribution_states(states, &mut grid, *size);

        // Add the states that have a box distribution.
        Self::add_box_distribution_states(states, &mut grid, *size);

        // Add the states that have a quantity distribution. They can overwrite states without a quantity distribution.
        Self::add_q_distribution_states(states, &mut grid, *size);

        let grid_next = grid.clone();

        Automaton {
            grid,
            grid_next,
            rules,
        }
    }

    fn add_p_distribution_states(states: &[State], grid: &mut Vec<Cell>, size: (usize, usize)) {
        let mut rng = rand::thread_rng();
        for x in 0..size.0 {
            for y in 0..size.1 {
                let index = get_index((x as isize, y as isize), size);
                let r_p: f64 = rng.gen();
                let mut lower_bound = 0.0;
                let mut upper_bound = 0.0;

                for (i, state) in states.iter().enumerate() {
                    if let StateDistribution::Proportion(p) = state.distribution {
                        upper_bound += p;
                        if r_p >= lower_bound && r_p < upper_bound {
                            grid[index].state = i;
                        }
                        lower_bound = upper_bound;
                    }
                }
            }
        }
    }

    fn add_box_distribution_states(states: &[State], grid: &mut Vec<Cell>, size: (usize, usize)) {
        for (i, state) in states.iter().enumerate() {
            if let StateDistribution::Box(x_box, y_box, width, height) = state.distribution {
                for x in x_box..(x_box + width) {
                    for y in y_box..(y_box + height) {
                        let index = get_index((x as isize, y as isize), size);
                        grid[index].state = i;
                    }
                }
            }
        }
    }

    fn add_q_distribution_states(states: &[State], grid: &mut Vec<Cell>, size: (usize, usize)) {
        let mut rng = rand::thread_rng();
        let mut positions_used = Vec::new();
        for (i, state) in states.iter().enumerate() {
            if let StateDistribution::Quantity(q) = state.distribution {
                let mut c = 0;
                while c < q {
                    let pos = (rng.gen_range(0, size.0), rng.gen_range(0, size.1));
                    if !positions_used.contains(&pos) {
                        let index = get_index((pos.0 as isize, pos.1 as isize), size);
                        grid[index].state = i;
                        positions_used.push(pos);
                        c += 1;
                    }
                }
            }
        }
    }

    pub fn tick(&mut self) {
        let rules = &self.rules;
        let grid = &self.grid;

        self.grid_next.par_iter_mut().for_each(|cell| {
            let position = get_position(cell.index_in_grid, rules.world_size);
            let mut rng = rand::thread_rng();
            for (state_origin, state_destination, conditions) in &rules.transitions {
                if state_origin == &grid[cell.index_in_grid].state && rules.evaluate_conditions(grid, position, conditions, &mut rng) {
                    cell.state = *state_destination;
                    break;
                }
            }
        });

        for index in 0..self.grid.len() {
            self.grid[index].state = self.grid_next[index].state;
        }
    }

    pub fn get_state(&self, x: isize, y: isize) -> usize {
        self.grid[get_index((x, y), self.rules.world_size)].state
    }

    pub fn get_colors(&self) -> Vec<(u8, u8, u8)> {
        self.rules.states.iter().map(|s| s.color).collect::<Vec<_>>()
    }
}

impl Rules {
    fn evaluate_conditions(&self, grid: &[Cell], position: (usize, usize), conditions: &[Vec<Condition>], rng: &mut ThreadRng) -> bool {
        match conditions.iter().find(|conjunction| self.evaluate_conjunction(grid, position, conjunction, rng)) {
            Some(_) => true,
            _ => false
        }
    }

    fn evaluate_conjunction(&self, grid: &[Cell], position: (usize, usize), conjunction: &[Condition], rng: &mut ThreadRng) -> bool {
        match conjunction.iter().find(|condition| !self.evaluate_condition(grid, position, condition, rng)) {
            Some(_) => false,
            _ => true
        }
    }

    fn evaluate_condition(&self, grid: &[Cell], position: (usize, usize), condition: &Condition, rng: &mut ThreadRng) -> bool {
        match condition {
            Condition::QuantityCondition(state, comp, quantity) => {
                let count = self.count_state_in_neighborhood(grid, position, *state);
                Self::evaluate_quantity_condition(count, *comp, *quantity)
            },
            Condition::NeighborCondition(neighbor, state) => {
                let (x, y) = (position.0 as isize, position.1 as isize);
                let index = Self::get_index_of_neighbor((x, y), *neighbor, self.world_size);
                self.is_state(grid[index].state, *state)
            },
            Condition::RandomCondition(proportion) => {
                let r: f64 = rng.gen();
                r < *proportion
            },
            Condition::True => true
        }
    }

    fn count_state_in_neighborhood(&self, grid: &[Cell], (x, y): (usize, usize), state: usize) -> u8 {
        let mut count: u8 = 0;
        for u in -1..2 {
            for v in -1..2 {
                if u != 0 || v != 0 {
                    let position = (x as isize + u, y as isize + v);
                    let index = get_index(position, self.world_size);
                    if self.is_state(grid[index].state, state) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn is_state(&self, state: usize, other_state: usize) -> bool {
        if state == other_state {
            return true;
        }
        if let Some(range) = &self.implicit_state_ranges[other_state] {
            return state >= range.start && state < range.len;
        }
        false
    }

    fn evaluate_quantity_condition(count: u8, comp: ComparisonOperator, quantity: u8) -> bool {
        match comp {
            ComparisonOperator::Greater => count > quantity,
            ComparisonOperator::Lesser => count < quantity,
            ComparisonOperator::GreaterOrEqual => count >= quantity,
            ComparisonOperator::LesserOrEqual => count <= quantity,
            ComparisonOperator::Equal => count == quantity,
            ComparisonOperator::Different => count != quantity
        }
    }

    fn get_index_of_neighbor((x, y): (isize, isize), neighbor: NeighborCell, size: (usize, usize)) -> usize {
        let neighbor_position = match neighbor {
            NeighborCell::A => (x - 1, y - 1),
            NeighborCell::B => (x, y - 1),
            NeighborCell::C => (x + 1, y - 1),
            NeighborCell::D => (x - 1, y),
            NeighborCell::E => (x + 1, y),
            NeighborCell::F => (x - 1, y + 1),
            NeighborCell::G => (x, y + 1),
            NeighborCell::H => (x + 1, y + 1)
        };
        get_index(neighbor_position, size)
    }
}

fn get_position(index: usize, size: (usize, usize)) -> (usize, usize) {
    assert!(index < size.0 * size.1,
            "The index {} is too big to be located in the matrix of size ({},{}).", index, size.0, size.1);
    (index % size.0, index / size.0)
}

fn get_index((x, y): (isize, isize), size: (usize, usize)) -> usize {
    tore_correction(y, size.1) * size.0 + tore_correction(x, size.0)
}

/// The world is a tore, so the value range can be )-inf; +inf(, and it will be mapped to (0; upper_bound-1).
fn tore_correction(value: isize, upper_bound: usize) -> usize {
    if value >= 0 {
        (value as usize) % upper_bound
    } else {
        // don't question my magic
        let signed_upper_bound = upper_bound as isize;
        let corrected = (signed_upper_bound + (value % signed_upper_bound)) % signed_upper_bound;
        corrected as usize
    }
}
