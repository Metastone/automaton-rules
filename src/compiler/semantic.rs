/// This module provides semantic analysis functions

use crate::compiler::parser;
use crate::compiler::parser::*;

pub enum StateDistribution {
    Proportion(f64),
    Quantity(usize),
    Default
}

pub struct State {
    pub id: usize,
    pub name: String,
    pub color: (u8, u8, u8), // 16M color
    pub distribution: StateDistribution
}

pub struct Rules {
    pub states: Vec<State>,
    pub transitions: Vec<Transition>
}

pub type Transition = (usize, usize, Vec<ConditionsConjunction>);

pub type ConditionsConjunction = Vec<Condition>;

pub enum Condition {
    QuantityCondition(usize, ComparisonOperator, u8),
    NeighborCondition(NeighborCell, usize),
    True
}

/// Parses the file and returns a data structure that represents the automaton's rules described in the file.
///
/// If it finds a lexical or syntax error, the parsing is stopped and the error is returned.
/// Otherwise, it performs a semantic analysis. If the semantic analysis fails, returns the list of semantic errors.
pub fn parse(file_name: &str) -> Result<Rules, Vec<String>> {
    match parser::parse(file_name) {
        Ok(ast) => semantic_analysis(&ast),
        Err(error) => Err(vec![error])
    }
}

fn semantic_analysis(ast: & Ast) -> Result<Rules, Vec<String>> {
    let mut errors = Vec::new();

    if let StateNode::Next(_) = ast {
        errors.push("You should specify at least one state.".to_string());
    }

    let (states, first_transition_node) = construct_states(ast);
    control_states_distribution(&states, &mut errors);
    let transitions = construct_transitions(first_transition_node, &states, &mut errors);

    match errors.len() {
        0 => Ok(Rules { states, transitions }),
        _ => Err(errors)
    }
}

fn construct_states<'a>(ast: &'a Ast) -> (Vec<State>, &'a TransitionNode) {
    let mut curr_state_node = ast;
    let first_transition_node: &TransitionNode;
    let mut states = Vec::new();
    let mut id = 0;
    loop {
        match curr_state_node {
            StateNode::State(name, red, green, blue, state_distribution_node) => {
                let (distribution, state_node) = match state_distribution_node {
                    StateDistributionNode::Proportion(proportion, state_node) => (StateDistribution::Proportion(*proportion), state_node.as_ref()),
                    StateDistributionNode::Quantity(quantity, state_node) => (StateDistribution::Quantity(*quantity), state_node.as_ref()),
                    StateDistributionNode::Default(state_node) => (StateDistribution::Default, state_node.as_ref())
                };
                states.push(State {
                    id,
                    name: name.clone(),
                    color: (red.clone(), green.clone(), blue.clone()),
                    distribution
                });
                id += 1;
                curr_state_node = state_node;
            },
            StateNode::Next(t) => {
                first_transition_node = &t;
                break;
            }
        }
    }
    (states, first_transition_node)
}

fn control_states_distribution(states: &Vec<State>, errors: &mut Vec<String>) {
    let proportions_sum = states.iter().fold(0.0, |sum, s|
        sum + match s.distribution {
            StateDistribution::Proportion(p) => p,
            _ => 0.0
        });
    if proportions_sum >= 1.0 {
        errors.push(format!("The sum of state's proportions must be lesser than 1.0, but it is currently {}.", proportions_sum));
    }

    let default_count = states.iter().filter(|s|
        match s.distribution {
            StateDistribution::Default => true,
            _ => false
        }).count();
    if default_count != 1 {
        errors.push(format!(
            "There must be exactly one default state (without a distribution specified), but there are currently {} of such states.",
            default_count));
    }
}

fn construct_transitions(first_transition_node: &TransitionNode, states: &Vec<State>, errors: &mut Vec<String>) -> Vec<Transition> {
    let mut curr_transition_node = first_transition_node;
    let mut transitions = Vec::new();
    loop {
        match curr_transition_node {
            TransitionNode::Transition(state_origin_name, state_destination_name, condition_node) => {
                let state_origin = match get_state_index(state_origin_name, &states) {
                    Some(index) => index,
                    _ => {
                        errors.push(transition_undefined_state_error(state_origin_name, state_destination_name, state_origin_name));
                        0   // whatever is the number here, it won't be used because an error occurred
                    }
                };
                let state_destination = match get_state_index(state_destination_name, &states) {
                    Some(index) => index,
                    _ => {
                        errors.push(transition_undefined_state_error(state_origin_name, state_destination_name, state_destination_name));
                        0   // whatever is the number here, it won't be used because an error occurred
                    }
                };
                let (transition_node, processed_condition) = construct_condition(condition_node, &states, errors);
                curr_transition_node = transition_node;
                transitions.push((state_origin, state_destination, processed_condition));
            },
            TransitionNode::End => {
                break;
            }
        }
    }
    transitions
}

fn get_state_index(state_name: & String, states: & Vec<State>) -> Option<usize> {
    states.into_iter().position(|s| &s.name == state_name)
}

fn construct_condition<'a>(root_condition_node: &'a ConditionNode,
                       states: &Vec<State>,
                       errors: &mut Vec<String>) -> (&'a TransitionNode, Vec<ConditionsConjunction>) {
    let mut processed_condition = Vec::new();
    let mut curr_condition_conjunction = Vec::new();
    let mut curr_condition_node = root_condition_node;
    let next_transition_node: &TransitionNode;
    loop {
        let (condition, next_condition_node) = match curr_condition_node {
            ConditionNode::QuantityCondition(state_name, comp_op, quantity, next_condition_node) => {
                let state = match get_state_index(state_name, states) {
                    Some(index) => index,
                    _ => {
                        errors.push(condition_undefined_state_error(state_name));
                        0   // whatever is the number here, it won't be used because an error occurred
                    }
                };
                (Condition::QuantityCondition(state, comp_op.clone(), quantity.clone()), next_condition_node)
            },
            ConditionNode::NeighborCondition(cell, state_name, next_condition_node) => {
                let state = match get_state_index(state_name, states) {
                    Some(index) => index,
                    _ => {
                        errors.push(condition_undefined_state_error(state_name));
                        0   // whatever is the number here, it won't be used because an error occurred
                    }
                };
                (Condition::NeighborCondition(cell.clone(), state), next_condition_node)
            },
            ConditionNode::True(next_condition_node) => {
               (Condition::True, next_condition_node)
            }
        };

        curr_condition_conjunction.push(condition);

        let condition_is_true = if let ConditionNode::True(_) = curr_condition_node { true } else { false };
        let conditions_before = curr_condition_conjunction.len() > 1 || processed_condition.len() > 0;
        let conditions_after = if let NextConditionNode::NextCondition(_,_) = next_condition_node { true } else { false };
        if condition_is_true && (conditions_before || conditions_after) {
            errors.push(condition_true_error());
        }

        match next_condition_node {
            NextConditionNode::NextCondition(bool_op, condition_node) => {
                curr_condition_node = condition_node.as_ref();
                if let BooleanOperator::Or = bool_op {
                    processed_condition.push(curr_condition_conjunction);
                    curr_condition_conjunction = Vec::new();
                }
            },
            NextConditionNode::NextTransition(t) => {
                next_transition_node = t.as_ref();
                if curr_condition_conjunction.len() > 0 {
                     processed_condition.push(curr_condition_conjunction);
                }
                break;
            }
        }
    }
    (next_transition_node, processed_condition)
}

fn transition_undefined_state_error(state_origin: & String,
                                    state_destination: & String,
                                    undefined: & String) -> String {
    format!("The transition '{} -> {}' refers to the state \"{}\", but it's not defined.", state_origin, state_destination, undefined)
}

fn condition_undefined_state_error(state_name: & String) -> String {
    format!("A condition refers to the state \"{}\", but it's not defined.", state_name)
}

fn condition_true_error() -> String {
    "The \"true\" condition should not be alone, not combined with other conditions.".to_string()
}

#[cfg(test)]
mod tests {
    use crate::compiler::semantic::parse;

    static BENCHMARK_FILE: &str = "resources/tests/compiler_benchmark.txt";
    static CONDITION_UNDEFINED_STATE_FILE: &str = "resources/tests/semantic_condition_undefined_state.txt";
    static NO_STATES_FILE: &str = "resources/tests/semantic_no_states.txt";
    static SEVERAL_ERRORS_FILE: &str = "resources/tests/semantic_several_errors.txt";
    static TRANSITION_UNDEFINED_STATE_FILE: &str = "resources/tests/semantic_transition_undefined_state.txt";
    static TRUE_ERROR_FILE: &str = "resources/tests/semantic_true_error.txt";
    static TWO_DEFAULT_STATES_FILE: &str = "resources/tests/semantic_two_default_states.txt";
    static WRONG_PROPORTIONS_FILE: &str = "resources/tests/semantic_wrong_proportions.txt";

    #[test]
    fn parse_benchmark_succeeds() {
        match parse(BENCHMARK_FILE) {
            Ok(_) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_condition_undefined_state_fails() {
        match parse(CONDITION_UNDEFINED_STATE_FILE) {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0], "A condition refers to the state \"happy\", but it\'s not defined.");
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_no_states_fails() {
        match parse(NO_STATES_FILE) {
            Err(errors) => {
                assert_eq!(errors.len(), 2);
                assert_eq!(errors[0], "You should specify at least one state.");
                assert_eq!(errors[1], "There must be exactly one default state (without a distribution specified), but there are currently 0 of such states.");
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_several_errors_fails() {
        match parse(SEVERAL_ERRORS_FILE) {
            Err(errors) => {
                assert_eq!(errors.len(), 9);
                assert_eq!(errors[0], "The transition \'alive -> dead\' refers to the state \"alive\", but it\'s not defined.");
                assert_eq!(errors[1], "The transition \'alive -> dead\' refers to the state \"dead\", but it\'s not defined.");
                assert_eq!(errors[2], "The \"true\" condition should not be alone, not combined with other conditions.");
                assert_eq!(errors[3], "A condition refers to the state \"alive\", but it\'s not defined.");
                assert_eq!(errors[4], "The transition \'dead -> alive\' refers to the state \"dead\", but it\'s not defined.");
                assert_eq!(errors[5], "The transition \'dead -> alive\' refers to the state \"alive\", but it\'s not defined.");
                assert_eq!(errors[6], "A condition refers to the state \"alive\", but it\'s not defined.");
                assert_eq!(errors[7], "A condition refers to the state \"dead\", but it\'s not defined.");
                assert_eq!(errors[8], "A condition refers to the state \"alive\", but it\'s not defined.");
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_transition_undefined_state_fails() {
        match parse(TRANSITION_UNDEFINED_STATE_FILE) {
            Err(errors) => {
                assert_eq!(errors.len(),  1);
                assert_eq!(errors[0], "The transition \'happy -> dead\' refers to the state \"happy\", but it\'s not defined.");
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_true_error_fails() {
        match parse(TRUE_ERROR_FILE) {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0], "The \"true\" condition should not be alone, not combined with other conditions.");
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_two_default_states_fails() {
        match parse(TWO_DEFAULT_STATES_FILE) {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0], "There must be exactly one default state (without a distribution specified), but there are currently 2 of such states.");
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_wrong_proportions_fails() {
        match parse(WRONG_PROPORTIONS_FILE) {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0], "The sum of state's proportions must be lesser than 1.0, but it is currently 1.1.");
            },
            _ => assert!(false)
        }
    }

}
