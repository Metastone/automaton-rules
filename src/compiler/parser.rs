/// This module provides syntax analysis functions

use crate::compiler::lexer::{Token, Lexer};

#[derive(Clone)]
pub enum ComparisonOperator {
    Greater,
    Lesser,
    GreaterOrEqual,
    LesserOrEqual,
    Equal,
    Different
}

#[derive(Clone)]
pub enum NeighborCell {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H
}

pub enum BooleanOperator {
    And,
    Or
}

pub enum NextConditionNode {
    NextCondition(BooleanOperator, Box<ConditionNode>),
    NextTransition(Box<TransitionNode>)
}

pub enum ConditionNode {
    QuantityCondition(String, ComparisonOperator, u8, NextConditionNode),
    NeighborCondition(NeighborCell, String, NextConditionNode),
    True(NextConditionNode)
}

pub enum TransitionNode {
    Transition(String, String, Box<ConditionNode>),
    End
}

pub enum StateDistributionNode {
    Proportion(f64, Box<StateNode>),
    Quantity(usize, Box<StateNode>),
    Default(Box<StateNode>)
}

pub enum StateNode {
    State(String, u8, u8, u8, StateDistributionNode),
    Next(TransitionNode)
}

pub type Ast = StateNode;

/// Parses the file to create an AST that matches the automaton description language grammar.
/// If an error occurs, the parsing is stopped and the error is returned.
///
/// Two main types of errors can be generated :
///     - lexical error if the error occurred in the lexical analyzer (lexer)
///     - syntax error if the file does not match the grammar
pub fn parse(file_name: &str) -> Result<Ast, String> {
    let mut lexer: Lexer;
    match Lexer::new(file_name) {
        Ok(l) => { lexer = l; },
        Err(io_error) => { return Err(format!("Cannot parse file {}. Cause : {:?}", file_name, io_error)); }
    };

    expect(&mut lexer, vec!["states"])?;
    expect(&mut lexer, vec!["{"])?;
    parse_state(&mut lexer)
}

fn parse_state(lexer: &mut Lexer) -> Result<StateNode, String> {
    let token = expect(lexer, vec!["(", "}"])?;
    if token == "(" {
        let state_name = expect_identifier(lexer)?;
        expect(lexer, vec![","])?;
        let red = expect_u8(lexer)?;
        expect(lexer, vec![","])?;
        let green = expect_u8(lexer)?;
        expect(lexer, vec![","])?;
        let blue = expect_u8(lexer)?;
        Ok(StateNode::State(state_name, red, green, blue, parse_state_distribution(lexer)?))
    } else {
        expect(lexer, vec!["transitions"])?;
        expect(lexer, vec!["{"])?;
        Ok(StateNode::Next(parse_transitions(lexer)?))
    }
}

fn parse_state_distribution(lexer: &mut Lexer) -> Result<StateDistributionNode, String> {
    let token = expect(lexer, vec![")", ","])?;
    if token == ")" {
        expect(lexer, vec![","])?;
        Ok(StateDistributionNode::Default(Box::new(parse_state(lexer)?)))
    } else {
        let token2 = expect(lexer, vec!["proportion", "quantity"])?;
        if token2 == "proportion" {
            let proportion = expect_proportion(lexer)?;
            expect(lexer, vec![")"])?;
            expect(lexer, vec![","])?;
            Ok(StateDistributionNode::Proportion(proportion, Box::new(parse_state(lexer)?)))
        } else {
            let quantity = expect_usize(lexer)?;
            expect(lexer, vec![")"])?;
            expect(lexer, vec![","])?;
            Ok(StateDistributionNode::Quantity(quantity, Box::new(parse_state(lexer)?)))
        }
    }
}

fn parse_transitions(lexer: &mut Lexer) -> Result<TransitionNode, String> {
    let token = expect(lexer, vec!["(", "}"])?;
    if token == "(" {
        let initial_state_name = expect_identifier(lexer)?;
        expect(lexer, vec![","])?;
        let next_state_name = expect_identifier(lexer)?;
        expect(lexer, vec![","])?;
        Ok(TransitionNode::Transition(initial_state_name, next_state_name, Box::new(parse_condition(lexer)?)))
    }
    else {
        Ok(TransitionNode::End)
    }
}

fn parse_condition(lexer: &mut Lexer) -> Result<ConditionNode, String> {
    let token = lexer.get_next_token()?;
    if token.str == "true" {
        Ok(ConditionNode::True(parse_next_condition(lexer)?))
    }
    else if let Some(neighbor_cell) = to_neighbor_cell(&token) {
        expect(lexer, vec!["is"])?;
        let state_name = expect_identifier(lexer)?;
        Ok(ConditionNode::NeighborCondition(neighbor_cell, state_name, parse_next_condition(lexer)?))
    }
    else if is_identifier(&token) {
        let comparison_operator = expect_comparison_operator(lexer)?;
        let number = expect_neighbor_number(lexer)?;
        Ok(ConditionNode::QuantityCondition(token.str, comparison_operator, number, parse_next_condition(lexer)?))
    }
    else {
        Err(format!("Expected either token \"true\", a neighbor cell identifier \
            (one of \"A\", \"B\", \"C\", \"D\", \"E\", \"F\", \"H\"), or an alphanumeric identifier, but found {}.", token))
    }
}

fn parse_next_condition(lexer: &mut Lexer) -> Result<NextConditionNode, String> {
    let token = lexer.get_next_token()?;
    if let Some(boolean_operator) = to_boolean_operator(&token) {
        Ok(NextConditionNode::NextCondition(boolean_operator, Box::new(parse_condition(lexer)?)))
    }
    else if token.str == ")" {
        expect(lexer, vec![","])?;
        Ok(NextConditionNode::NextTransition(Box::new(parse_transitions(lexer)?)))
    }
    else {
        Err(format!("Expected either a boolean operator \"&&\", \"||\" or a \")\" token, found {}.", token))
    }
}

/// Return the next token if it's one of the expected tokens, or raises an error.
fn expect(lexer: &mut Lexer, expected: Vec<&str>) -> Result<String, String> {
    let mut expected_as_sentence = String::new();
    let token = lexer.get_next_token()?;
    for i in 0..expected.len() {
        if token.str == expected[i] {
            return Ok(token.str);
        }
        if i != 0 {
            expected_as_sentence.push_str(" or ");
        }
        expected_as_sentence.push_str("\"");
        expected_as_sentence.push_str(expected[i]);
        expected_as_sentence.push_str("\"");
    }
    Err(format!("Expected {}, found {}.", expected_as_sentence, token))
}

/// Return the next token if it's an alphanumeric identifier, or raises an error.
fn expect_identifier(lexer: &mut Lexer) -> Result<String, String> {
    let token = lexer.get_next_token()?;
    if is_identifier(&token) {
        Ok(token.str)
    }
    else {
        Err(format!("Expected an alphanumeric identifier, found {}.", token))
    }
}

fn is_identifier(token: & Token) -> bool {
    token.str.chars().all(|c| c.is_ascii_alphanumeric())
        && !token.str.chars().all(|c| c.is_ascii_digit())
}

/// Return the next token translated into an integer between 0 and 255 if possible, or raises an error.
fn expect_u8(lexer: &mut Lexer) -> Result<u8, String> {
    let token = lexer.get_next_token()?;
    match token.str.parse::<u8>() {
        Ok(number) => Ok(number),
        Err(_) => Err(format!("Expected an integer between 0 and 255, found {}.", token))
    }
}

/// Return the next token translated into an integer between 0 and 8 if possible, or raises an error.
fn expect_neighbor_number(lexer: &mut Lexer) -> Result<u8, String> {
    let token = lexer.get_next_token()?;
    if let Ok(number) = token.str.parse::<u8>() {
        if number <= 8 {
            return Ok(number);
        }
    }
    Err(format!("Expected an integer between 0 and 8, found {}.", token))
}

/// Return the next token translated into a floating number between 0 and 1 if possible, or raises an error.
fn expect_proportion(lexer: &mut Lexer) -> Result<f64, String> {
    let token = lexer.get_next_token()?;
    if let Ok(number) = token.str.parse::<f64>() {
        if number >= 0.0 && number <= 1.0 {
            return Ok(number)
        }
    }
    Err(format!("Expected a floating number between 0 and 1, found {}.", token))
}

/// Return the next token translated into an unsigned integer if possible, or raises an error.
fn expect_usize(lexer: &mut Lexer) -> Result<usize, String> {
    let token = lexer.get_next_token()?;
    match token.str.parse::<usize>() {
        Ok(number) => Ok(number),
        Err(_) => Err(format!("Expected an unsigned integer, found {}.", token))
    }
}

/// Return a comparison operator if the next token represents one, or raises an error.
fn expect_comparison_operator(lexer: &mut Lexer) -> Result<ComparisonOperator, String> {
    let token = lexer.get_next_token()?;
    match token.str.as_str() {
        "<" => Ok(ComparisonOperator::Lesser),
        ">" => Ok(ComparisonOperator::Greater),
        "<=" => Ok(ComparisonOperator::LesserOrEqual),
        ">=" => Ok(ComparisonOperator::GreaterOrEqual),
        "==" => Ok(ComparisonOperator::Equal),
        "!=" => Ok(ComparisonOperator::Different),
        _ => Err(format!("Expected one of \"<\", \">\", \"<=\", \">=\", \"==\", or \"!=\" tokens, found {}.", token))
    }
}

/// Translate the token into a boolean operator, if possible.
fn to_boolean_operator(token: & Token) -> Option<BooleanOperator> {
    match token.str.as_str() {
        "&&" => Some(BooleanOperator::And),
        "||" => Some(BooleanOperator::Or),
        _ => None
    }
}

/// Translate the token into a neighbor cell identifier, if possible.
fn to_neighbor_cell(token: & Token) -> Option<NeighborCell> {
    match token.str.as_str() {
        "A" => Some(NeighborCell::A),
        "B" => Some(NeighborCell::B),
        "C" => Some(NeighborCell::C),
        "D" => Some(NeighborCell::D),
        "E" => Some(NeighborCell::E),
        "F" => Some(NeighborCell::F),
        "G" => Some(NeighborCell::G),
        "H" => Some(NeighborCell::H),
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::parser::parse;

    static BENCHMARK_FILE: &str = "resources/tests/compiler_benchmark.txt";
    static NON_EXISTING_FILE: &str = "resources/tests/does_not_exist.txt";
    static COND_ERROR_FILE: &str = "resources/tests/parser_condition_error.txt";
    static EXPECT_COMP_OP_FILE: &str = "resources/tests/parser_expected_comparison_operator.txt";
    static EXPECT_ID_FILE: &str = "resources/tests/parser_expected_identifier.txt";
    static EXPECT_IS_FILE: &str = "resources/tests/parser_expected_is_token.txt";
    static EXPECT_NEIGHBOR_NB_FILE: &str = "resources/tests/parser_expected_neighbor_number.txt";
    static EXPECT_PROPORTION_FILE: &str = "resources/tests/parser_expected_proportion.txt";
    static EXPECT_U8_FILE: &str = "resources/tests/parser_expected_u8.txt";
    static EXPECT_USIZE_FILE: &str = "resources/tests/parser_expected_usize.txt";
    static NEXT_COND_ERROR_FILE: &str = "resources/tests/parser_next_condition_error.txt";
    static NO_STATES_FILE: &str = "resources/tests/parser_no_states_keyword.txt";

    #[test]
    fn parse_benchmark_succeeds() {
        match parse(BENCHMARK_FILE) {
            Ok(_) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_non_existing_file_fails() {
         match parse(NON_EXISTING_FILE) {
            Err(error) => {
                assert!(error.contains("Cannot parse file resources/tests/does_not_exist.txt. Cause : "));
                assert!(error.contains("No such file or directory"));
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_condition_error_fails() {
        match parse(COND_ERROR_FILE) {
            Err(error) => assert_eq!(error, "Expected either token \"true\", a neighbor cell identifier (one of \"A\", \"B\", \"C\", \"D\", \"E\", \"F\", \"H\"), \
            or an alphanumeric identifier, but found \"3153\" - line 7, column 22."),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_expect_comp_operator_fails() {
        match parse(EXPECT_COMP_OP_FILE) {
            Err(error) => assert_eq!(error, "Expected one of \"<\", \">\", \"<=\", \">=\", \"==\", or \"!=\" tokens, found \"plouf\" - line 7, column 29."),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_expect_identifier_fails() {
         match parse(EXPECT_ID_FILE) {
            Err(error) => assert_eq!(error, "Expected an alphanumeric identifier, found \"51566\" - line 3, column 10."),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_expect_is_token_fails() {
         match parse(EXPECT_IS_FILE) {
            Err(error) => assert_eq!(error, "Expected \"is\", found \"plouf\" - line 8, column 39."),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_expect_neighbor_number_fails() {
         match parse(EXPECT_NEIGHBOR_NB_FILE) {
            Err(error) => assert_eq!(error, "Expected an integer between 0 and 8, found \"22\" - line 7, column 28."),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_expect_proportion_fails() {
        match parse(EXPECT_PROPORTION_FILE) {
            Err(error) => assert_eq!(error, "Expected a floating number between 0 and 1, found \"2.5\" - line 2, column 41."),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_expect_u8_fails() {
         match parse(EXPECT_U8_FILE) {
            Err(error) => assert_eq!(error, "Expected an integer between 0 and 255, found \"260\" - line 2, column 15."),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_expect_usize_fails() {
        match parse(EXPECT_USIZE_FILE) {
            Err(error) => assert_eq!(error, "Expected an unsigned integer, found \"yolo\" - line 4, column 42."),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_next_condition_error_fails() {
         match parse(NEXT_COND_ERROR_FILE) {
            Err(error) => assert_eq!(error, "Expected either a boolean operator \"&&\", \"||\" or a \")\" token, found \"dead\" - line 8, column 46."),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_no_states_keyword_fails() {
         match parse(NO_STATES_FILE) {
            Err(error) => assert_eq!(error, "Expected \"states\", found \"plouf\" - line 1, column 5."),
            _ => assert!(false)
        }
    }
}
