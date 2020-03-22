use crate::compiler::lexer::{Token, Lexer};

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

pub enum Condition {
    QuantityCondition(String, ComparisonOperator, u8, NextCondition),
    PositionCondition(NeighborCell, String, NextCondition),
    True(NextCondition)
}

pub enum Transition {
    Transition(String, String, Box<Condition>),
    End
}

pub enum State {
    State(String, u8, u8, u8, Box<State>),
    Next(Transition)
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

fn parse_condition(lexer: &mut Lexer) -> Result<Condition, String> {
    let token = lexer.get_next_token()?;
    if token.str == "true" {
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
        Ok(Condition::QuantityCondition(token.str, comparison_operator, number, parse_next_condition(lexer)?))
    }
    else {
        Err(format!("Expected either token \"true\", a neighbor cell identifier \
            (one of \"A\", \"B\", \"C\", \"D\", \"E\", \"F\", \"H\"), or an alphanumeric identifier, but found {}.", token))
    }
}

fn parse_next_condition(lexer: &mut Lexer) -> Result<NextCondition, String> {
    let token = lexer.get_next_token()?;
    if let Some(boolean_operator) = to_boolean_operator(&token) {
        Ok(NextCondition::NextCondition(boolean_operator, Box::new(parse_condition(lexer)?)))
    }
    else if token.str == ")" {
        expect(lexer, vec![","])?;
        Ok(NextCondition::NextTransition(Box::new(parse_transitions(lexer)?)))
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

/// Return success if the next token is 'is', or raises an error.
fn expect_is(lexer: &mut Lexer) -> Result<(), String> {
    let token = lexer.get_next_token()?;
    match token.str.as_str() {
        "is" => Ok(()),
        _ => Err(format!("Expected \"is\" token, found {}.", token))
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

    static BENCHMARK_FILE: &str = "resources/tests/parser_benchmark.txt";
    static NON_EXISTING_FILE: &str = "resources/tests/does_not_exist.txt";
    static COND_ERROR_FILE: &str = "resources/tests/parser_condition_error.txt";
    static EXPECT_COMP_OP_FILE: &str = "resources/tests/parser_expected_comparison_operator.txt";
    static EXPECT_ID_FILE: &str = "resources/tests/parser_expected_identifier.txt";
    static EXPECT_IS_FILE: &str = "resources/tests/parser_expected_is_token.txt";
    static EXPECT_NEIGHBOR_NB_FILE: &str = "resources/tests/parser_expected_neighbor_number.txt";
    static EXPECT_U8_FILE: &str = "resources/tests/parser_expected_u8.txt";
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
            Err(error) => assert_eq!(error, "Expected \"is\" token, found \"plouf\" - line 8, column 39."),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_expect_neigbhor_number_fails() {
         match parse(EXPECT_NEIGHBOR_NB_FILE) {
            Err(error) => assert_eq!(error, "Expected an integer between 0 and 8, found \"22\" - line 7, column 28."),
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
