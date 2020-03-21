use crate::compiler::lexer::Lexer;

pub enum ComparisonOperator {
    Greater,
    Lesser,
    GreaterOrEqual,
    LesserOrEqual,
    Equal,
    Different
}

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

pub enum NextCondition {
    NextCondition(BooleanOperator, Box<Condition>),
    NextTransition(Box<Transition>)
}

fn parse_next_condition(lexer: &mut Lexer) -> Result<NextCondition, String> {
    let token = lexer.get_next_token()?;
    if let Some(boolean_operator) = to_boolean_operator(&token) {
        Ok(NextCondition::NextCondition(boolean_operator, Box::new(parse_condition(lexer)?)))
    }
    else if token == ")" {
        expect(lexer, vec![","])?;
        Ok(NextCondition::NextTransition(Box::new(parse_transitions(lexer)?)))
    }
    else {
        Err(format!("Expected either a boolean operator \"&&\", \"||\" or a \")\" token, found {}", token))
    }
}

pub enum Condition {
    QuantityCondition(String, ComparisonOperator, u8, NextCondition),
    PositionCondition(NeighborCell, String, NextCondition),
    True(NextCondition)
}

fn parse_condition(lexer: &mut Lexer) -> Result<Condition, String> {
    let token = lexer.get_next_token()?;
    if token == "true" {
        Ok(Condition::True(parse_next_condition(lexer)?))
    }
    else if let Some(neighbor_cell) = to_neighbor_cell(&token) {
        expect_is(lexer)?;
        let state_name = expect_identifier(lexer)?;
        Ok(Condition::PositionCondition(neighbor_cell, state_name, parse_next_condition(lexer)?))
    }
    else if is_identifier(&token) {
        let comparison_operator = expect_comparison_operator(lexer)?;
        let number = expect_neighbor_number(lexer)?;
        Ok(Condition::QuantityCondition(token, comparison_operator, number, parse_next_condition(lexer)?))
    }
    else {
        Err(format!("Expected either token \"true\", a neighbor cell identifier \
            (one of \"A\", \"B\", \"C\", \"D\", \"E\", \"F\", \"H\"), or an alphanumeric identifier, but found {}", token))
    }
}

pub enum Transition {
    Transition(String, String, Box<Condition>),
    End
}

fn parse_transitions(lexer: &mut Lexer) -> Result<Transition, String> {
    let token = expect(lexer, vec!["(", "}"])?;
    if token == "(" {
        let initial_state_name = expect_identifier(lexer)?;
        expect(lexer, vec![","])?;
        let next_state_name = expect_identifier(lexer)?;
        expect(lexer, vec![","])?;
        Ok(Transition::Transition(initial_state_name, next_state_name, Box::new(parse_condition(lexer)?)))
    }
    else {
        Ok(Transition::End)
    }
}

pub enum State {
    State(String, u8, u8, u8, Box<State>),
    Next(Transition)
}

fn parse_state(lexer: &mut Lexer) -> Result<State, String> {
    let token = expect(lexer, vec!["(", "}"])?;
    if token == "(" {
        let state_name = expect_identifier(lexer)?;
        expect(lexer, vec![","])?;
        let red = expect_u8(lexer)?;
        expect(lexer, vec![","])?;
        let green = expect_u8(lexer)?;
        expect(lexer, vec![","])?;
        let blue = expect_u8(lexer)?;
        expect(lexer, vec![")"])?;
        expect(lexer, vec![","])?;
        Ok(State::State(state_name, red, green, blue, Box::new(parse_state(lexer)?)))
    } else {
        expect(lexer, vec!["transitions"])?;
        expect(lexer, vec!["{"])?;
        Ok(State::Next(parse_transitions(lexer)?))
    }
}

type Ast = State;

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

/// Return the next token if it's one of the expected tokens, or raises an error.
fn expect(lexer: &mut Lexer, expected: Vec<&str>) -> Result<String, String> {
    let token = lexer.get_next_token()?;
    for i in 0..expected.len() {
        if token == expected[i] {
            return Ok(token);
        }
    }
    Err(format!("Expected {:?}, found {}", expected, token))
}

/// Return the next token if it's an alphanumeric identifier, or raises an error.
fn expect_identifier(lexer: &mut Lexer) -> Result<String, String> {
    let token = lexer.get_next_token()?;
    if is_identifier(&token) {
        Ok(token)
    }
    else {
        Err(format!("Expected an alphanumeric identifier, found {}", token))
    }
}

fn is_identifier(token: & String) -> bool {
    token.chars().all(|c| c.is_ascii_alphanumeric()) && !token.chars().all(|c| c.is_ascii_digit())
}

/// Return the next token translated into an integer between 0 and 255 if possible, or raises an error.
fn expect_u8(lexer: &mut Lexer) -> Result<u8, String> {
    let token = lexer.get_next_token()?;
    match token.parse::<u8>() {
        Ok(number) => Ok(number),
        Err(_) => Err(format!("Expected an integer between 0 and 255, found {}", token))
    }
}

/// Return the next token translated into an integer between 0 and 8 if possible, or raises an error.
fn expect_neighbor_number(lexer: &mut Lexer) -> Result<u8, String> {
    let token = lexer.get_next_token()?;
    if let Ok(number) = token.parse::<u8>() {
        if number <= 8 {
            return Ok(number);
        }
    }
    Err(format!("Expected an integer between 0 and 8, found {}", token))
}

/// Return success if the next token is 'is', or raises an error.
fn expect_is(lexer: &mut Lexer) -> Result<(), String> {
    let token = lexer.get_next_token()?;
    match token.as_str() {
        "is" => Ok(()),
        _ => Err(format!("Expected \"is\" token, found {}", token))
    }
}

/// Return a comparison operator if the next token represents one, or raises an error.
fn expect_comparison_operator(lexer: &mut Lexer) -> Result<ComparisonOperator, String> {
    let token = lexer.get_next_token()?;
    match token.as_str() {
        "<" => Ok(ComparisonOperator::Lesser),
        ">" => Ok(ComparisonOperator::Greater),
        "<=" => Ok(ComparisonOperator::LesserOrEqual),
        ">=" => Ok(ComparisonOperator::GreaterOrEqual),
        "==" => Ok(ComparisonOperator::Equal),
        "!=" => Ok(ComparisonOperator::Different),
        _ => Err(format!("Expected one of \"<\", \">\", \"<=\", \">=\", \"==\", or \"!=\" tokens, found {}", token))
    }
}

/// Translate the token into a boolean operator, if possible.
fn to_boolean_operator(token: & String) -> Option<BooleanOperator> {
    match token.as_str() {
        "&&" => Some(BooleanOperator::And),
        "||" => Some(BooleanOperator::Or),
        _ => None
    }
}

/// Translate the token into a neighbor cell identifier, if possible.
fn to_neighbor_cell(token: & String) -> Option<NeighborCell> {
    match token.as_str() {
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
