/// This module provides semantic analysis functions

use crate::compiler::parser;
use crate::compiler::parser::*;
use std::collections::{
    HashMap,
    hash_map::RandomState
};

pub struct Automaton {
    pub states: HashMap<String, (u8, u8, u8)>,
    pub transitions: Vec<Transition>
}

type Transition = (String, String, Vec<ConditionsConjunction>);

type ConditionsConjunction = Vec<Condition>;

pub enum Condition {
    QuantityCondition(String, ComparisonOperator, u8),
    NeighborCondition(NeighborCell, String),
    True
}

/// Parses the file and returns a data structure that represents the automaton described in the file.
///
/// If it finds a lexical or syntax error, the parsing is stopped and the error is returned.
/// Otherwise, it performs a semantic analysis. If the semantic analysis fails, returns the list of semantic errors.
pub fn parse(file_name: &str) -> Result<Automaton, Vec<String>> {
    match parser::parse(file_name) {
        Ok(ast) => semantic_analysis(&ast),
        Err(error) => Err(vec![error])
    }
}

fn semantic_analysis(ast: & Ast) -> Result<Automaton, Vec<String>> {
    let mut errors = Vec::new();

    if let StateNode::Next(_) = ast {
        errors.push("You should specify at least one state.".to_string());
    }

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
                    errors.push(transition_undefined_state_error(state_origin, state_destination, state_origin));
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
        0 => Ok(Automaton { states, transitions }),
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
            println!("len conj {}, len all {}, conditions_after {}", curr_condition_conjunction.len(), processed_condition.len(), conditions_after);
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
    format!("The transition '{} -> {}' refers to the state {}, but it's not defined.", state_origin, state_destination, undefined)
}

fn condition_undefined_state_error(state_name: & String) -> String {
    format!("A condition refers to the state {}, but it's not defined.", state_name)
}

fn condition_true_error() -> String {
    "The \"true\" condition should not be alone, not combined with other conditions.".to_string()
}
