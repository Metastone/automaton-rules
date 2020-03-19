use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};

static DELIMITORS: [char; 5] = ['{', '}', '(', ')', ','];
static SINGLE_CHAR_OPERATORS: [char; 2] = ['<', '>'];
static TWO_CHAR_OPERATORS: [&str; 6] = ["&&", "||", "==", "!=", "<=", ">="];
static OPERATOR_FIRST_CHARS: [char; 6] = ['&', '|', '=', '!', '<', '>'];

pub struct Lexer<'a> {
    reader: BufReader<File>,
    file_name: &'a str
}

impl<'a> Lexer<'a> {
    pub fn new(file_name: &str) -> Result<Lexer, io::Error> {
        let file = File::open(file_name)?;
        let reader = BufReader::new(file);
        Ok(Lexer {
            reader,
            file_name
        })
    }

    pub fn get_next_token(&mut self) -> Result<String, String> {
        // Read until a not-whitespace parameter is found.
        let mut c = self.read_char()?;
        while c.is_ascii_whitespace() && c != '\u{0}' {
            c = self.read_char()?;
        }

        // The token is a single delimitor character.
        if DELIMITORS.contains(&c) {
            return Ok(c.to_string());
        }

        // The token seems to be an operator.
        if OPERATOR_FIRST_CHARS.contains(&c) {
            return self.get_operator_token(c);
        }

        // The token should be a number or an alpha-numeric identifier (that doesn't start with a number).
        self.get_number_or_id_token(c)
    }

    fn get_operator_token(&mut self, c: char) -> Result<String, String> {
        let mut token = String::new();
        token.push(c);
        let expected_char = match c {
            '!' => '=',
            '<' => '=',
            '>' => '=',
            _ => c
        };
        let c2 = self.read_char()?;
        token.push(c2);

        // The token is a two-characters operator
        if c2 == expected_char {
            return Ok(token);
        }
        // The token is a single character operator ('<' or '>')
        else if (c == '<' || c == '>')
            && (c2.is_ascii_whitespace()
                || c2 == '\u{0}'
                || c2.is_ascii_alphanumeric()
                || DELIMITORS.contains(&c2)
                || OPERATOR_FIRST_CHARS.contains(&c2)) {
            token.pop();
            if let Err(error) = self.reader.seek(SeekFrom::Current(-1)) {
                return Err(format!("Could not get token. Cause : {:?}", error));
            }
            return Ok(token);
        }
        // The token starts as an operator but not one
        else {
            return Err(format!("Invalid token {}. Note : recognized operators are {:?} and {:?}.", token, SINGLE_CHAR_OPERATORS, TWO_CHAR_OPERATORS));
        }
    }

    fn get_number_or_id_token(&mut self, first_char: char) -> Result<String, String> {
        let is_token_number = first_char.is_ascii_digit();
        let is_token_identifier = first_char.is_ascii_alphabetic();
        let mut rewind_one_char = false;

        let mut token = String::new();
        let mut c = first_char;

        while !c.is_ascii_whitespace() && c != '\u{0}' {
            token.push(c);

            if is_token_number && !c.is_ascii_digit() {
                if DELIMITORS.contains(&c) || OPERATOR_FIRST_CHARS.contains(&c) {
                    rewind_one_char = true;
                    break;
                } else {
                    return Err(format!("Invalid token {}. It starts with a digit but is not a number.", token));
                }
            }

            if is_token_identifier && !c.is_ascii_alphanumeric() {
                if DELIMITORS.contains(&c) || OPERATOR_FIRST_CHARS.contains(&c) {
                    rewind_one_char = true;
                    break;
                } else {
                    return Err(format!("Invalid token {}. It contains illegal characters.", token));
                }
            }

            c = self.read_char()?;
        }

        // No token found and we reached end-of-file
        if token.len() == 0 && c == '\u{0}' {
            return Err("No token available, end of file reached.".to_string());
        }

        // The last character is nor part of the token, we just have to remove it and we are good.
        if rewind_one_char {
            token.pop();
            if let Err(error) = self.reader.seek(SeekFrom::Current(-1)) {
                return Err(format!("Could not get token. Cause : {:?}", error));
            }
        }

        // Token is a valid number or identifier
        Ok(token)
    }

    fn read_char(&mut self) -> Result<char, String> {
        let mut buffer = [0; 1];
        match self.reader.read(&mut buffer) {
            Ok(_) => return Ok(buffer[0] as char),
            Err(e) => return Err(format!("Cannot read character from file {}. Cause : {:?}", self.file_name, e))
        }
    }
}
