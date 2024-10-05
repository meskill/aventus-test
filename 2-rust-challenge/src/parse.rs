//! Module that converts iterator of Token
//! into Prefix (Polish) Notation iterator
//! that later could be evaluated by eval module

use std::{cmp::Ordering, fmt::Display};

use super::tokens::{Group, Result as TokenizerResult, Token, TokenizerError};

#[derive(Debug, PartialEq)]
pub enum ParserError {
    TokenizerError(TokenizerError),
    EmptyExpr,
    UnbalancedGroup(Option<Token>),
    OperatorExpected(Option<Token>),
    OperandExpected {
        token: Option<Token>,
        operator: Option<Token>,
    },
}

impl From<TokenizerError> for ParserError {
    fn from(error: TokenizerError) -> Self {
        Self::TokenizerError(error)
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TokenizerError(error) => write!(f, "{error}"),
            Self::EmptyExpr => write!(f, "Expression is empty"),
            Self::UnbalancedGroup(_) => write!(f, "Unbalanced brackets"),
            Self::OperatorExpected(_) => write!(f, "Expected operator"),
            Self::OperandExpected { .. } => write!(f, "Expected operand"),
        }
    }
}

impl std::error::Error for ParserError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TokenizerError(error) => Some(error),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, ParserError>;

#[derive(Default, Debug)]
enum State {
    #[default]
    Start,
    Operand,
    OperatorOrEnd,
}

#[derive(Default, Debug)]
pub struct ExprAst {
    stack: Vec<Token>,
    state: State,
    group_nesting_index: u32,
    last_group_token: Option<Token>,
}

impl<'token> IntoIterator for &'token ExprAst {
    type Item = &'token Token;

    type IntoIter = std::iter::Rev<std::slice::Iter<'token, Token>>;

    fn into_iter(self) -> Self::IntoIter {
        self.stack.iter().rev()
    }
}

impl ExprAst {
    fn parse(
        &mut self,
        tokens_iter: &mut impl Iterator<Item = TokenizerResult<Token>>,
    ) -> Result<()> {
        self.parse_group(tokens_iter)?;

        if self.group_nesting_index > 0 {
            return Err(ParserError::UnbalancedGroup(self.last_group_token.take()));
        }

        if let State::Start = self.state {
            return Err(ParserError::EmptyExpr);
        }

        Ok(())
    }

    fn parse_group(
        &mut self,
        tokens_iter: &mut impl Iterator<Item = TokenizerResult<Token>>,
    ) -> Result<()> {
        let mut operator_stack = vec![];

        while let Some(token) = tokens_iter.next() {
            let token = token?;

            match self.state {
                State::Start | State::Operand => match token {
                    Token::Number(_) => {
                        self.stack.push(token);
                        self.state = State::OperatorOrEnd
                    }
                    Token::Operator(ref operator) => {
                        if operator.arity() > 1 {
                            return Err(ParserError::OperandExpected {
                                token: Some(token),
                                operator: operator_stack.pop(),
                            });
                        }

                        let last_token = operator_stack.last();

                        if let Some(Token::Operator(prev_op)) = last_token {
                            if operator == prev_op {
                                return Err(ParserError::OperandExpected {
                                    token: Some(token),
                                    operator: operator_stack.pop(),
                                });
                            }
                        }

                        operator_stack.push(token);
                        self.state = State::Operand;
                    }
                    Token::Group(Group::Open) => {
                        self.group_nesting_index += 1;
                        self.state = State::Start;
                        self.last_group_token = Some(token);
                        self.parse_group(tokens_iter)?;
                    }
                    Token::Group(Group::Close) => {
                        return Err(if operator_stack.is_empty() {
                            if self.group_nesting_index > 0 {
                                ParserError::EmptyExpr
                            } else {
                                ParserError::UnbalancedGroup(Some(token))
                            }
                        } else {
                            ParserError::OperandExpected {
                                token: Some(token),
                                operator: operator_stack.pop(),
                            }
                        })
                    }
                },
                State::OperatorOrEnd => match token {
                    Token::Number(_) | Token::Group(Group::Open) => {
                        return Err(ParserError::OperatorExpected(Some(token)))
                    }
                    Token::Operator(ref operator) => {
                        while let Some(prev_op) = operator_stack.last() {
                            if let Token::Operator(prev_op) = prev_op {
                                if operator.cmp(prev_op) == Ordering::Less {
                                    break;
                                }
                            }

                            self.stack
                                .push(operator_stack.pop().expect("prev_op is Some"));
                        }

                        operator_stack.push(token);
                        self.state = State::Operand;
                    }
                    Token::Group(Group::Close) => {
                        if self.group_nesting_index == 0 {
                            return Err(ParserError::UnbalancedGroup(Some(token)));
                        }

                        self.group_nesting_index -= 1;
                        self.state = State::OperatorOrEnd;
                        break;
                    }
                },
            }
        }

        if let State::Operand = self.state {
            return Err(ParserError::OperandExpected {
                token: None,
                operator: operator_stack.pop(),
            });
        }

        while let Some(operator) = operator_stack.pop() {
            self.stack.push(operator);
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct ExprParser {
    inner_parser: ExprAst,
}

impl ExprParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(
        &mut self,
        tokens_iter: &mut impl Iterator<Item = TokenizerResult<Token>>,
    ) -> Result<impl Iterator<Item = &Token>> {
        self.inner_parser.parse(tokens_iter)?;

        Ok(self.inner_parser.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{Number, Operator, TokenIterator};

    macro_rules! assert_parse {
                ($expr:literal, $( $token:expr ),* $(,)?) => {
                    let mut parser = ExprParser::new();
                    let parsed = parser.parse(&mut TokenIterator::from($expr)).unwrap();
                    assert_eq!(parsed.collect::<Vec<_>>(), vec![$(&$token, )*])
                };
            }

    macro_rules! assert_parse_error {
        ($expr: literal, $err: expr) => {
            let mut parser = ExprParser::new();
            let err = parser
                .parse(&mut TokenIterator::from($expr))
                .map(|_| ())
                .unwrap_err();
            assert_eq!(err, $err)
        };
    }

    #[test]
    fn unit_expr() {
        assert_parse!("2", Token::Number(Number::Int(2)));
        assert_parse!("3.7", Token::Number(Number::Float(3.7)));
    }

    #[test]
    fn simple_expr() {
        assert_parse!(
            "2 a  3",
            Token::Operator(Operator::Add),
            Token::Number(Number::Int(3)),
            Token::Number(Number::Int(2))
        );
    }

    #[test]
    fn priority_expr() {
        assert_parse!(
            "2 a 2 b 3",
            Token::Operator(Operator::Sub),
            Token::Number(Number::Int(3)),
            Token::Operator(Operator::Add),
            Token::Number(Number::Int(2)),
            Token::Number(Number::Int(2))
        );

        assert_parse!(
            "2 a 2 c 3",
            Token::Operator(Operator::Mul),
            Token::Number(Number::Int(3)),
            Token::Operator(Operator::Add),
            Token::Number(Number::Int(2)),
            Token::Number(Number::Int(2))
        );

        assert_parse!(
            "2 c 2 a 3",
            Token::Operator(Operator::Add),
            Token::Number(Number::Int(3)),
            Token::Operator(Operator::Mul),
            Token::Number(Number::Int(2)),
            Token::Number(Number::Int(2))
        );

        assert_parse!(
            "2 a 2 c 3 c b2",
            Token::Operator(Operator::Mul),
            Token::Operator(Operator::Neg),
            Token::Number(Number::Int(2)),
            Token::Operator(Operator::Mul),
            Token::Number(Number::Int(3)),
            Token::Operator(Operator::Add),
            Token::Number(Number::Int(2)),
            Token::Number(Number::Int(2)),
        );
    }

    #[test]
    fn grouping() {
        assert_parse!("e1f", Token::Number(Number::Int(1)));
        assert_parse!("eee1fff", Token::Number(Number::Int(1)));
        assert_parse!(
            "bebe1ff",
            Token::Operator(Operator::Neg),
            Token::Operator(Operator::Neg),
            Token::Number(Number::Int(1)),
        );

        assert_parse!(
            "2 c 3 a e2 a 3 f c 5.1", // 2 * 3 + ( 2 + 3 ) * 5.1
            Token::Operator(Operator::Mul),
            Token::Number(Number::Float(5.1)),
            Token::Operator(Operator::Add),
            Token::Operator(Operator::Add),
            Token::Number(Number::Int(3)),
            Token::Number(Number::Int(2)),
            Token::Operator(Operator::Mul),
            Token::Number(Number::Int(3)),
            Token::Number(Number::Int(2)),
        );
    }

    #[test]
    fn unbalanced_brackets() {
        assert_parse_error!(
            "f",
            ParserError::UnbalancedGroup(Some(Token::Group(Group::Close)))
        );
        assert_parse_error!(
            "5f",
            ParserError::UnbalancedGroup(Some(Token::Group(Group::Close)))
        );
        assert_parse_error!(
            "e",
            ParserError::UnbalancedGroup(Some(Token::Group(Group::Open)))
        );
        assert_parse_error!(
            "e 3",
            ParserError::UnbalancedGroup(Some(Token::Group(Group::Open)))
        );
        assert_parse_error!(
            "1 a 2 c e3 b 2 f a f",
            ParserError::OperandExpected {
                token: Some(Token::Group(Group::Close)),
                operator: Some(Token::Operator(Operator::Add))
            }
        );
        assert_parse_error!(
            "1 a 2 c e3 b 2 f c e",
            ParserError::UnbalancedGroup(Some(Token::Group(Group::Open)))
        );
        assert_parse_error!(
            "e 3 a e2 a 2f",
            ParserError::UnbalancedGroup(Some(Token::Group(Group::Open)))
        );
        assert_parse_error!(
            "e 3 a e2 a 2f f f",
            ParserError::UnbalancedGroup(Some(Token::Group(Group::Close)))
        );

        assert_parse_error!("", ParserError::EmptyExpr);

        assert_parse_error!("ef", ParserError::EmptyExpr);
    }

    #[test]
    fn missing_operand() {
        assert_parse_error!(
            "a",
            ParserError::OperandExpected {
                token: Some(Token::Operator(Operator::Add)),
                operator: None
            }
        );
        assert_parse_error!(
            "c 3 b 2",
            ParserError::OperandExpected {
                token: Some(Token::Operator(Operator::Mul)),
                operator: None
            }
        );
        assert_parse_error!(
            "2 b",
            ParserError::OperandExpected {
                token: None,
                operator: Some(Token::Operator(Operator::Sub))
            }
        );
        assert_parse_error!(
            "2 a e3 b 2f c",
            ParserError::OperandExpected {
                token: None,
                operator: Some(Token::Operator(Operator::Mul))
            }
        );
        assert_parse_error!(
            "2 c d 2",
            ParserError::OperandExpected {
                token: Some(Token::Operator(Operator::Div)),
                operator: Some(Token::Operator(Operator::Mul))
            }
        );
        assert_parse_error!(
            "b ",
            ParserError::OperandExpected {
                token: None,
                operator: Some(Token::Operator(Operator::Neg))
            }
        );
        assert_parse_error!(
            " 2 a b",
            ParserError::OperandExpected {
                token: None,
                operator: Some(Token::Operator(Operator::Neg))
            }
        );
        assert_parse_error!(
            " 2 a ebf",
            ParserError::OperandExpected {
                token: Some(Token::Group(Group::Close)),
                operator: Some(Token::Operator(Operator::Neg))
            }
        );
        assert_parse_error!(
            "b b 2",
            ParserError::OperandExpected {
                token: Some(Token::Operator(Operator::Sub)),
                operator: Some(Token::Operator(Operator::Neg))
            }
        );
        assert_parse_error!(
            "2 c b b 2",
            ParserError::OperandExpected {
                token: Some(Token::Operator(Operator::Sub)),
                operator: Some(Token::Operator(Operator::Neg))
            }
        );
    }

    #[test]
    fn missing_operator() {
        assert_parse_error!(
            "2 3",
            ParserError::OperatorExpected(Some(Token::Number(Number::Int(3))))
        );
        assert_parse_error!(
            "e2 a3 f e5c6f",
            ParserError::OperatorExpected(Some(Token::Group(Group::Open)))
        );
    }
}
