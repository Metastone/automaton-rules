/// This module provides semantic analysis functions

use crate::compiler::parser;
use crate::compiler::parser::*;
use std::collections::{
    HashMap,
    hash_map::RandomState
};

pub struct Rules {
    pub initial_state: String,
    pub states: HashMap<String, (u8, u8, u8)>,
    pub transitions: Vec<Transition>
}

pub type Transition = (String, String, Vec<ConditionsConjunction>);

pub type ConditionsConjunction = Vec<Condition>;

pub enum Condition {
    QuantityCondition(String, ComparisonOperator, u8),
    NeighborCondition(NeighborCell, String),
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

    let initial_state = match ast {
        StateNode::State(state, _, _, _, _) => state.clone(),
        StateNode::Next(_) => {
            errors.push("You should specify at least one state.".to_string());
            "".to_string()
        }
    };

    let mut states = HashMap::new();
    let mut curr_state_node = ast;
    let mut curr_transition_node: &TransitionNode;

    loop {
        match curr_state_node {
            StateNode::State(name, red, green, blue, state_node) => {
                states.insert(name.clone(), (red.clone(), green.clone(), blue.clone()));
                curr_state_node = state_node.as_ref();
            },
            StateNode::Next(t) => {
                curr_transition_node = &t;
                break;
            }
        }
    }

    let mut transitions = Vec::new();

    loop {
        match curr_transition_node {
            TransitionNode::Transition(state_origin, state_destination, condition_node) => {
                if !states.contains_key(state_origin) {
                    errors.push(transition_undefined_state_error(state_origin, state_destination, state_origin));
                }
                if !states.contains_key(state_destination) {
                    errors.push(transition_undefined_state_error(state_origin, state_destination, state_destination));
                }
                let (transition_node, processed_condition) =
                    construct_condition(condition_node, &mut states, &mut errors);
                transitions.push((state_origin.clone(), state_destination.clone(), processed_condition));
                curr_transition_node = transition_node;
            },
            TransitionNode::End => {
                break;
            }
        }
    }

    match errors.len() {
        0 => Ok(Rules { initial_state, states, transitions }),
        _ => Err(errors)
    }
}

fn construct_condition<'a>(root_condition_node: &'a ConditionNode,
                       states: &mut HashMap<String, (u8, u8, u8), RandomState>,
                       errors: &mut Vec<String>) -> (&'a TransitionNode, Vec<ConditionsConjunction>) {
    let mut processed_condition = Vec::new();
    let mut curr_condition_conjunction = Vec::new();
    let mut curr_condition_node = root_condition_node;
    let next_transition_node: &TransitionNode;
    loop {
        let (condition, next_condition_node) = match curr_condition_node {
            ConditionNode::QuantityCondition(state_name, comp_op, quantity, next_condition_node) => {
               if !states.contains_key(state_name) {
                    errors.push(condition_undefined_state_error(state_name));
                }
                (Condition::QuantityCondition(state_name.clone(), comp_op.clone(), quantity.clone()), next_condition_node)
            },
            ConditionNode::NeighborCondition(cell, state_name, next_condition_node) => {
               if !states.contains_key(state_name) {
                    errors.push(condition_undefined_state_error(state_name))
                }
                (Condition::NeighborCondition(cell.clone(), state_name.clone()), next_condition_node)
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
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0], "You should specify at least one state.");
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_several_errors_fails() {
        match parse(SEVERAL_ERRORS_FILE) {
            Err(errors) => {
                assert_eq!(errors.len(), 10);
                assert_eq!(errors[0], "You should specify at least one state.");
                assert_eq!(errors[1], "The transition \'alive -> dead\' refers to the state \"alive\", but it\'s not defined.");
                assert_eq!(errors[2], "The transition \'alive -> dead\' refers to the state \"dead\", but it\'s not defined.");
                assert_eq!(errors[3], "The \"true\" condition should not be alone, not combined with other conditions.");
                assert_eq!(errors[4], "A condition refers to the state \"alive\", but it\'s not defined.");
                assert_eq!(errors[5], "The transition \'dead -> alive\' refers to the state \"dead\", but it\'s not defined.");
                assert_eq!(errors[6], "The transition \'dead -> alive\' refers to the state \"alive\", but it\'s not defined.");
                assert_eq!(errors[7], "A condition refers to the state \"alive\", but it\'s not defined.");
                assert_eq!(errors[8], "A condition refers to the state \"dead\", but it\'s not defined.");
                assert_eq!(errors[9], "A condition refers to the state \"alive\", but it\'s not defined.");
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
}
