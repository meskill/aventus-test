//! Module to parse string into stream of tokens
//! i.e. operands, operators and brackets

use std::{
    cmp::Ordering,
    error::Error,
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
    str::Chars,
};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Operator {
    Neg,
    Add,
    Sub,
    Mul,
    Div,
}

/// Priority of the operation
type Priority = u8;

/// Number of arguments used by operation
pub type Arity = u8;

fn get_operator_priority(operator: &Operator) -> Priority {
    match operator {
        Operator::Neg => 0,
        // There is no difference in priority for operations
        _ => 1,
    }
}

impl Operator {
    pub fn arity(&self) -> Arity {
        match self {
            Operator::Neg => 1,
            _ => 2,
        }
    }
}

impl Ord for Operator {
    fn cmp(&self, other: &Self) -> Ordering {
        get_operator_priority(self).cmp(&get_operator_priority(other))
    }
}

impl PartialOrd for Operator {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Group {
    Open,
    Close,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Number {
    Int(i32),
    Float(f64),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    Operator(Operator),
    Group(Group),
    Number(Number),
}

#[derive(Debug, PartialEq, Eq)]
pub enum NumberParseErrorKind {
    Int(ParseIntError),
    Float(ParseFloatError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenizerError {
    UnknownToken(char),
    NumberParseError { kind: NumberParseErrorKind },
}

impl Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownToken(token) => write!(f, "Unknown token `{token}` in the stream"),
            Self::NumberParseError { .. } => write!(f, "Unable to parse number"),
        }
    }
}

impl Error for TokenizerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let TokenizerError::NumberParseError { kind } = self {
            match kind {
                NumberParseErrorKind::Int(err) => Some(err),
                NumberParseErrorKind::Float(err) => Some(err),
            }
        } else {
            None
        }
    }
}

impl From<ParseIntError> for TokenizerError {
    fn from(value: ParseIntError) -> Self {
        Self::NumberParseError {
            kind: NumberParseErrorKind::Int(value),
        }
    }
}

impl From<ParseFloatError> for TokenizerError {
    fn from(value: ParseFloatError) -> Self {
        Self::NumberParseError {
            kind: NumberParseErrorKind::Float(value),
        }
    }
}

pub type Result<T> = std::result::Result<T, TokenizerError>;

pub struct TokenIterator<'stream> {
    stream: Chars<'stream>,
    input: Option<char>,
    expect_for_neg: bool,
}

impl<'stream> TokenIterator<'stream> {
    pub fn new(stream: Chars<'stream>) -> Self {
        Self {
            stream,
            input: None,
            expect_for_neg: true,
        }
    }

    fn exhaust_whitespace(&mut self) {
        for input in self.stream.by_ref() {
            if input != ' ' {
                self.input = Some(input);

                return;
            }
        }
    }

    fn exhaust_number(&mut self, first_digit: char) -> Result<Number> {
        let mut had_dot = first_digit == '.';
        let mut digits = vec![first_digit];

        for input in self.stream.by_ref() {
            match input {
                '0'..='9' => digits.push(input),
                '.' => {
                    had_dot = true;
                    digits.push(input);
                }
                _ => {
                    self.input = Some(input);
                    break;
                }
            }
        }

        let digits = String::from_iter(digits);

        if had_dot {
            Ok(Number::Float(digits.parse::<f64>()?))
        } else {
            Ok(Number::Int(digits.parse::<i32>()?))
        }
    }
}

impl<'stream> Iterator for TokenIterator<'stream> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_none() || self.input == Some(' ') {
            self.exhaust_whitespace();
        }

        let Some(input) = self.input.take() else {
            return None;
        };

        let result = match input {
            'a' => Token::Operator(Operator::Add),
            'b' => {
                if self.expect_for_neg {
                    Token::Operator(Operator::Neg)
                } else {
                    Token::Operator(Operator::Sub)
                }
            }
            'c' => Token::Operator(Operator::Mul),
            'd' => Token::Operator(Operator::Div),
            'e' => Token::Group(Group::Open),
            'f' => Token::Group(Group::Close),
            '0'..='9' | '.' => match self.exhaust_number(input) {
                Ok(num) => Token::Number(num),
                Err(err) => return Some(Err(err)),
            },
            _ => return Some(Err(TokenizerError::UnknownToken(input))),
        };

        self.expect_for_neg = match &result {
            Token::Operator(operator) => *operator != Operator::Neg,
            Token::Group(Group::Open) => true,
            _ => false,
        };

        Some(Ok(result))
    }
}

impl<'stream> From<&'stream str> for TokenIterator<'stream> {
    fn from(s: &'stream str) -> Self {
        TokenIterator::new(s.chars())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_tokens {
        ($str:literal) => {
            assert_eq!(TokenIterator::from($str).collect::<Vec<_>>(), vec![])
        };

		($str:literal, $( $token:expr ),* $(,)?) => {
			assert_eq!(TokenIterator::from($str).collect::<Vec<_>>(), vec![$($token),*])
		};
	}

    #[test]
    fn empty_string() {
        assert_tokens!("");
    }

    #[test]
    fn single_token() {
        assert_tokens!("a", Ok(Token::Operator(Operator::Add)));
        assert_tokens!("d", Ok(Token::Operator(Operator::Div)));
        assert_tokens!("e", Ok(Token::Group(Group::Open)));
        assert_tokens!("1", Ok(Token::Number(Number::Int(1))));
        assert_tokens!("1.25", Ok(Token::Number(Number::Float(1.25))));
        assert_tokens!(
            "b3.8",
            Ok(Token::Operator(Operator::Neg)),
            Ok(Token::Number(Number::Float(3.8)))
        );
        assert_tokens!(".5", Ok(Token::Number(Number::Float(0.5))));
        assert_tokens!("5.", Ok(Token::Number(Number::Float(5.0))))
    }

    #[test]
    fn complex_neg() {
        assert_tokens!(
            "ebeb1ff", // (-(-1))
            Ok(Token::Group(Group::Open)),
            Ok(Token::Operator(Operator::Neg)),
            Ok(Token::Group(Group::Open)),
            Ok(Token::Operator(Operator::Neg)),
            Ok(Token::Number(Number::Int(1))),
            Ok(Token::Group(Group::Close)),
            Ok(Token::Group(Group::Close)),
        );

        assert_tokens!(
            "b e 2f b e b 3f", // - ( 2) - ( - 3)
            Ok(Token::Operator(Operator::Neg)),
            Ok(Token::Group(Group::Open)),
            Ok(Token::Number(Number::Int(2))),
            Ok(Token::Group(Group::Close)),
            Ok(Token::Operator(Operator::Sub)),
            Ok(Token::Group(Group::Open)),
            Ok(Token::Operator(Operator::Neg)),
            Ok(Token::Number(Number::Int(3))),
            Ok(Token::Group(Group::Close))
        );
    }

    #[test]
    fn list_of_tokens() {
        assert_tokens!(
            " 2 a    3", // 2 +   3
            Ok(Token::Number(Number::Int(2))),
            Ok(Token::Operator(Operator::Add)),
            Ok(Token::Number(Number::Int(3)))
        );

        assert_tokens!(
            " 2.3 a e 2.9 b 3.5 c e 2.1 d 2.9 b 3253252.12f a 3f a 212", // 2.3 + ( 2.9 - 3.5 * (2.1 / 2.9 - 3253252.12) + 3 ) + 212
            Ok(Token::Number(Number::Float(2.3))),
            Ok(Token::Operator(Operator::Add)),
            Ok(Token::Group(Group::Open)),
            Ok(Token::Number(Number::Float(2.9))),
            Ok(Token::Operator(Operator::Sub)),
            Ok(Token::Number(Number::Float(3.5))),
            Ok(Token::Operator(Operator::Mul)),
            Ok(Token::Group(Group::Open)),
            Ok(Token::Number(Number::Float(2.1))),
            Ok(Token::Operator(Operator::Div)),
            Ok(Token::Number(Number::Float(2.9))),
            Ok(Token::Operator(Operator::Sub)),
            Ok(Token::Number(Number::Float(3253252.12))),
            Ok(Token::Group(Group::Close)),
            Ok(Token::Operator(Operator::Add)),
            Ok(Token::Number(Number::Int(3))),
            Ok(Token::Group(Group::Close)),
            Ok(Token::Operator(Operator::Add)),
            Ok(Token::Number(Number::Int(212)))
        )
    }

    #[test]
    fn wrong_single_token() {
        let parse_float_error = "..".parse::<f32>().unwrap_err();

        assert_tokens!(":", Err(TokenizerError::UnknownToken(':')));
        assert_tokens!("+", Err(TokenizerError::UnknownToken('+')));
        assert_tokens!("g", Err(TokenizerError::UnknownToken('g')));
        assert_tokens!(
            "2213.2132.233",
            Err(TokenizerError::NumberParseError {
                kind: NumberParseErrorKind::Float(parse_float_error)
            })
        );
        assert_tokens!(
            "2 + a",
            Ok(Token::Number(Number::Int(2))),
            Err(TokenizerError::UnknownToken('+')),
            Ok(Token::Operator(Operator::Add))
        );
        assert_tokens!(
            ": a e3 a 2f",
            Err(TokenizerError::UnknownToken(':')),
            Ok(Token::Operator(Operator::Add)),
            Ok(Token::Group(Group::Open)),
            Ok(Token::Number(Number::Int(3))),
            Ok(Token::Operator(Operator::Add)),
            Ok(Token::Number(Number::Int(2))),
            Ok(Token::Group(Group::Close))
        )
    }
}
