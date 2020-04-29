use crate::compiler::semantic::{State, Rules, Condition, StateDistribution, ConditionsConjunction};
use crate::compiler::parser::{NeighborCell, ComparisonOperator};
use rand::{Rng, rngs::ThreadRng};

pub struct Automaton {
    grid: Vec<usize>,
    grid_next: Vec<usize>,
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
        let mut grid = vec![default_state; size.0 * size.1];

        // Add the states that have a proportion distribution.
        Self::add_p_distribution_states(states, &mut grid, &size);

        // Add the states that have a box distribution.
        Self::add_box_distribution_states(states, &mut grid, &size);

        // Add the states that have a quantity distribution. They can overwrite states without a quantity distribution.
        Self::add_q_distribution_states(states, &mut grid, &size);

        let grid_next = grid.clone();

        Automaton {
            grid,
            grid_next,
            rules,
        }
    }

    fn add_p_distribution_states(states: &[State], grid: &mut Vec<usize>, size: &(usize, usize)) {
        let mut rng = rand::thread_rng();
        for x in 0..size.0 {
            for y in 0..size.1 {
                let index = Self::get_index(x as isize, y as isize, size);
                let r_p: f64 = rng.gen();
                let mut lower_bound = 0.0;
                let mut upper_bound = 0.0;

                for (i, state) in states.iter().enumerate() {
                    if let StateDistribution::Proportion(p) = state.distribution {
                        upper_bound += p;
                        if r_p >= lower_bound && r_p < upper_bound {
                            grid[index] = i;
                        }
                        lower_bound = upper_bound;
                    }
                }
            }
        }
    }

    fn add_box_distribution_states(states: &[State], grid: &mut Vec<usize>, size: &(usize, usize)) {
        for (i, state) in states.iter().enumerate() {
            if let StateDistribution::Box(x_box, y_box, width, height) = state.distribution {
                for x in x_box..(x_box + width) {
                    for y in y_box..(y_box + height) {
                        let index = Self::get_index(x as isize, y as isize, size);
                        grid[index] = i;
                    }
                }
            }
        }
    }

    fn add_q_distribution_states(states: &[State], grid: &mut Vec<usize>, size: &(usize, usize)) {
        let mut rng = rand::thread_rng();
        let mut positions_used = Vec::new();
        for (i, state) in states.iter().enumerate() {
            if let StateDistribution::Quantity(q) = state.distribution {
                let mut c = 0;
                while c < q {
                    let pos = (rng.gen_range(0, size.0), rng.gen_range(0, size.1));
                    if !positions_used.contains(&pos) {
                        let index = Self::get_index(pos.0 as isize, pos.1 as isize, size);
                        grid[index] = i;
                        positions_used.push(pos);
                        c += 1;
                    }
                }
            }
        }
    }

    pub fn tick(&mut self, rng: &mut ThreadRng) {
        for x in 0..self.rules.world_size.0 {
            for y in 0..self.rules.world_size.1 {
                let index = Self::get_index(x as isize, y as isize, &self.rules.world_size);
                let state = self.grid[index];
                for (state_origin, state_destination, conditions) in &self.rules.transitions {
                    if state_origin == &state && self.evaluate_conditions(x, y, conditions, rng) {
                        self.grid_next[index] = *state_destination;
                        break;
                    }
                }
            }
        }

        for x in 0..self.rules.world_size.0 {
            for y in 0..self.rules.world_size.1 {
                let index = Self::get_index(x as isize, y as isize, &self.rules.world_size);
                self.grid[index] = self.grid_next[index];
            }
        }
    }

    fn evaluate_conditions(& self, x: usize, y: usize, conditions: &[ConditionsConjunction], rng: &mut ThreadRng) -> bool {
        match conditions.iter().find(|conjunction| self.evaluate_conjunction(x, y, conjunction, rng)) {
            Some(_) => true,
            _ => false
        }
    }

    fn evaluate_conjunction(& self, x: usize, y: usize, conjunction: &ConditionsConjunction, rng: &mut ThreadRng) -> bool {
        match conjunction.iter().find(|condition| !self.evaluate_condition(x, y, condition, rng)) {
            Some(_) => false,
            _ => true
        }
    }

    fn evaluate_condition(& self, x: usize, y: usize, condition: &Condition, rng: &mut ThreadRng) -> bool {
        match condition {
            Condition::QuantityCondition(state, comp, quantity) => {
                let count = self.count_state_in_neighborhood(x, y, *state);
                Self::evaluate_quantity_condition(count, *comp, *quantity)
            },
            Condition::NeighborCondition(neighbor, state) => {
                let index = Self::get_index_of_neighbor(x as isize, y as isize, *neighbor, &self.rules.world_size);
                self.is_state(self.grid[index], *state)
            },
            Condition::RandomCondition(proportion) => {
                let r: f64 = rng.gen();
                r < *proportion
            },
            Condition::True => true
        }
    }

    fn count_state_in_neighborhood(& self, x: usize, y: usize, state: usize) -> u8 {
        let mut count: u8 = 0;
        for u in -1..2 {
            for v in -1..2 {
                if u != 0 || v != 0 {
                    let index =  Self::get_index(x as isize + u, y as isize + v, &self.rules.world_size);
                    if self.is_state(self.grid[index], state) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn is_state(& self, state: usize, other_state: usize) -> bool {
        if state == other_state {
            return true;
        }
        if let Some(range) = &self.rules.implicit_state_ranges[other_state] {
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

    fn get_index_of_neighbor(x: isize, y: isize, neighbor: NeighborCell, size: &(usize, usize)) -> usize {
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
        Self::get_index(x_n, y_n, size)
    }

    fn get_index(x: isize, y: isize, size: &(usize, usize)) -> usize {
        Self::tore_correction(y, size.1) * size.0 + Self::tore_correction(x, size.0)
    }

    /// The world is a tore, so the value range can be )-inf; +inf(, and it will be mapped to (0; upper_bound-1).
    fn tore_correction(value: isize, upper_bound: usize) -> usize {
        if value >= 0 {
            (value as usize) % upper_bound
        }
        else {
            // don't question my magic
            let signed_upper_bound = upper_bound as isize;
            let corrected = (signed_upper_bound + (value % signed_upper_bound)) % signed_upper_bound;
            corrected as usize
        }
    }

    pub fn get_state(&self, x: isize, y: isize) -> usize {
        self.grid[Self::get_index(x, y, &self.rules.world_size)]
    }

    pub fn get_colors(&self) -> Vec<(u8, u8, u8)> {
        self.rules.states.iter().map(|s| s.color).collect::<Vec<_>>()
    }
}
