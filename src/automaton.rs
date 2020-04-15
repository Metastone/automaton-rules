use crate::compiler::semantic::{State, Rules, Condition, StateDistribution, ConditionsConjunction};
use crate::compiler::parser::{NeighborCell, ComparisonOperator};
use rand::Rng;

pub struct Automaton {
    size: (usize, usize),
    grid: Vec<usize>,
    grid_next: Vec<usize>,
    rules: Rules
}

impl Automaton {
    pub fn new(rules: Rules) -> Automaton {
        let size = (200, 50);
        let states = &rules.states;

        // Initialize grid with default state.
        let default_state = (&rules).states.iter()
            .find(|s| match s.distribution {
                StateDistribution::Default => true,
                _ => false
            })
            .unwrap().id;
        let mut grid = vec![default_state; size.0 * size.1];

        // Add the states that have a proportion distribution.
        Self::add_p_distribution_states(states, &mut grid, &size);

        // Add the states that have a quantity distribution. They can overwrite states without a quantity distribution.
        Self::add_q_distribution_states(states, &mut grid, &size);

        let grid_next = grid.clone();

        Automaton {
            size,
            grid,
            grid_next,
            rules
        }
    }

    fn add_p_distribution_states(states: &Vec<State>, grid: &mut Vec<usize>, size: &(usize, usize)) {
        let mut rng = rand::thread_rng();
        for x in 0..size.0 {
            for y in 0..size.1 {
                let index = y * size.0 + x;
                let r_p: f64 = rng.gen();
                let mut lower_bound = 0.0;
                let mut upper_bound = 0.0;

                for i in 0..states.len() {
                    if let StateDistribution::Proportion(p) = states[i].distribution {
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

    fn add_q_distribution_states(states: &Vec<State>, grid: &mut Vec<usize>, size: &(usize, usize)) {
        let mut rng = rand::thread_rng();
        let mut positions_used = Vec::new();
        for i in 0..states.len() {
            if let StateDistribution::Quantity(q) = states[i].distribution {
                let mut c = 0;
                while c < q {
                    let pos = (rng.gen_range(0, size.0), rng.gen_range(0, size.1));
                    if !positions_used.contains(&pos) {
                        let index = pos.1 * size.0 + pos.0;
                        grid[index] = i;
                        positions_used.push(pos);
                        c += 1;
                    }
                }
            }
        }
    }

    pub fn tick(&mut self) {
        for x in 0..self.size.0 {
            for y in 0..self.size.1 {
                let index = self.get_index(x as isize, y as isize);
                let state = self.grid[index];
                for (state_origin, state_destination, conditions) in &self.rules.transitions {
                    if state_origin == &state {
                        if self.evaluate_conditions(x, y, conditions) {
                            self.grid_next[index] = state_destination.clone();
                        }
                    }
                }
            }
        }

        for x in 0..self.size.0 {
            for y in 0..self.size.1 {
                let index = self.get_index(x as isize, y as isize);
                self.grid[index] = self.grid_next[index];
            }
        }
    }

    fn evaluate_conditions(& self, x: usize, y: usize, conditions: &Vec<ConditionsConjunction>) -> bool {
        match conditions.iter().find(|conjunction| self.evaluate_conjunction(x, y, conjunction)) {
            Some(_) => true,
            _ => false
        }
    }

    fn evaluate_conjunction(& self, x: usize, y: usize, conjunction: &ConditionsConjunction) -> bool {
        match conjunction.iter().find(|condition| !self.evaluate_condition(x, y, condition)) {
            Some(_) => false,
            _ => true
        }
    }

    fn evaluate_condition(& self, x: usize, y: usize, condition: &Condition) -> bool {
        match condition {
            Condition::QuantityCondition(state, comp, quantity) => {
                let count = self.count_state_in_neighborhood(x, y, state);
                Self::evaluate_quantity_condition(&count, comp, quantity)
            },
            Condition::NeighborCondition(neighbor, state) => {
                let index = self.get_index_of_neighbor(x as isize, y as isize, neighbor);
                &self.grid[index] == state
            },
            Condition::True => true
        }
    }

    fn count_state_in_neighborhood(& self, x: usize, y: usize, state: &usize) -> u8 {
        let mut count: u8 = 0;
        for u in -1..2 {
            for v in -1..2 {
                if u != 0 || v != 0 {
                    let index =  self.get_index(x as isize + u, y as isize + v);
                    if &self.grid[index] == state {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn evaluate_quantity_condition(count: &u8, comp: &ComparisonOperator, quantity: &u8) -> bool {
        match comp {
            ComparisonOperator::Greater => count > quantity,
            ComparisonOperator::Lesser => count < quantity,
            ComparisonOperator::GreaterOrEqual => count >= quantity,
            ComparisonOperator::LesserOrEqual => count <= quantity,
            ComparisonOperator::Equal => count == quantity,
            ComparisonOperator::Different => count != quantity
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
            // don't question my magic
            let signed_upper_bound = upper_bound as isize;
            let corrected = (signed_upper_bound + (value % signed_upper_bound)) % signed_upper_bound;
            corrected as usize
        }
    }

    pub fn get_state(&self, x: isize, y: isize) -> usize {
        self.grid[self.get_index(x, y)]
    }

    pub fn get_colors(&self) -> Vec<(u8, u8, u8)> {
        self.rules.states.iter().map(|s| s.color).collect::<Vec<_>>()
    }
}
