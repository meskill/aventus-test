//! Module to evaluate parsed stream of tokens in
//! prefix notation and generates single output

use super::tokens::*;

#[derive(Debug, PartialEq)]
pub enum CalculationError {
    ZeroDivision,
}

impl std::fmt::Display for CalculationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroDivision => write!(f, "division by zero"),
        }
    }
}

impl std::error::Error for CalculationError {}

#[derive(Debug, PartialEq)]
pub enum EvalError {
    UnexpectedToken(Token),
    UnexpectedEndOfInput,
    UnconsumedToken(Token),
    CalculationError(CalculationError),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken(_) => write!(f, "Unexpected token"),
            Self::UnexpectedEndOfInput => write!(f, "Input stream has ended unexpectedly"),
            Self::UnconsumedToken(_) => write!(f, "Expression was calculated, but the stream contains more elements that were ignored"),
            Self::CalculationError(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for EvalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CalculationError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<CalculationError> for EvalError {
    fn from(error: CalculationError) -> Self {
        Self::CalculationError(error)
    }
}

pub type Result<T> = std::result::Result<T, EvalError>;

const ZERO: f64 = 0.0;

#[derive(Default)]
pub struct Evaluator;

impl Evaluator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn eval<'token>(&self, tokens: &mut impl Iterator<Item = &'token Token>) -> Result<f64> {
        let value = self.eval_inner(tokens)?;

        if let Some(token) = tokens.next() {
            return Err(EvalError::UnconsumedToken(token.clone()));
        }

        Ok(value)
    }

    fn eval_inner<'token>(&self, tokens: &mut impl Iterator<Item = &'token Token>) -> Result<f64> {
        let Some(token) = tokens.next() else {
            return Err(EvalError::UnexpectedEndOfInput);
        };

        match token {
            Token::Number(Number::Float(num)) => Ok(*num),
            Token::Number(Number::Int(num)) => Ok(*num as f64),
            Token::Operator(operator) => {
                let arity = operator.arity();
                let right_arg = if arity > 0 {
                    self.eval_inner(tokens)?
                } else {
                    ZERO
                };
                let left_arg = if arity > 1 {
                    self.eval_inner(tokens)?
                } else {
                    ZERO
                };

                Ok(match operator {
                    Operator::Neg => -right_arg,
                    Operator::Add => left_arg + right_arg,
                    Operator::Sub => left_arg - right_arg,
                    Operator::Mul => left_arg * right_arg,
                    Operator::Div => {
                        if right_arg == ZERO {
                            return Err(CalculationError::ZeroDivision.into());
                        }

                        left_arg / right_arg
                    }
                })
            }
            _ => Err(EvalError::UnexpectedToken(token.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{Number, Operator, Token};

    macro_rules! assert_eval {
        ($result: literal $(, $expr: expr)*) => {
            let evaluator = Evaluator::new();
            let tokens = vec![$($expr,)*];
            assert_eq!(evaluator.eval(&mut tokens.iter()).unwrap(), $result)
        };
    }

    macro_rules! assert_eval_error {
        ($err: expr $(, $expr: expr)*) => {
            let evaluator = Evaluator::new();
            let tokens = vec![$($expr,)*];
            assert_eq!(evaluator.eval(&mut tokens.iter()).unwrap_err(), $err)
        };
    }

    #[test]
    fn simple_expr() {
        assert_eval!(
            3.0,
            Token::Operator(Operator::Add),
            Token::Number(Number::Int(1)),
            Token::Number(Number::Int(2))
        );

        assert_eval!(
            5.0,
            Token::Operator(Operator::Mul),
            Token::Number(Number::Int(2)),
            Token::Number(Number::Float(2.5))
        );
    }

    #[test]
    fn complex_expr() {
        // (2 + 2 * 2 - 4 + 50 + (10 - 30)) / (3 + 2.8 * 2 - (0.3 * 2))
        assert_eval!(
            4.0,
            // () / ()
            Token::Operator(Operator::Div),
            // (2+...)
            Token::Operator(Operator::Sub),
            Token::Operator(Operator::Mul),
            Token::Number(Number::Int(2)),
            Token::Number(Number::Float(0.3)),
            Token::Operator(Operator::Add),
            Token::Operator(Operator::Mul),
            Token::Number(Number::Float(2.8)),
            Token::Number(Number::Int(2)),
            Token::Number(Number::Int(3)),
            // (3+...)
            Token::Operator(Operator::Add),
            // (10 - 30)
            Token::Operator(Operator::Sub),
            Token::Number(Number::Int(30)),
            Token::Number(Number::Int(10)),
            Token::Operator(Operator::Add),
            Token::Number(Number::Int(50)),
            Token::Operator(Operator::Sub),
            Token::Number(Number::Int(4)),
            Token::Operator(Operator::Add),
            Token::Operator(Operator::Mul),
            Token::Number(Number::Int(2)),
            Token::Number(Number::Int(2)),
            Token::Number(Number::Int(2))
        );
    }

    #[test]
    fn empty_expr() {
        assert_eval_error!(EvalError::UnexpectedEndOfInput);
    }

    #[test]
    fn unexpected_end() {
        assert_eval_error!(
            EvalError::UnexpectedEndOfInput,
            Token::Operator(Operator::Add),
            Token::Number(Number::Int(2))
        );

        assert_eval_error!(
            EvalError::UnexpectedEndOfInput,
            Token::Operator(Operator::Neg)
        );
    }

    #[test]
    fn unconsumed_tokens() {
        assert_eval_error!(
            EvalError::UnconsumedToken(Token::Number(Number::Int(3))),
            Token::Operator(Operator::Add),
            Token::Number(Number::Int(1)),
            Token::Number(Number::Int(2)),
            Token::Number(Number::Int(3))
        );
        assert_eval_error!(
            EvalError::UnconsumedToken(Token::Number(Number::Int(2))),
            Token::Number(Number::Int(1)),
            Token::Number(Number::Int(2))
        );
    }

    #[test]
    fn division_by_zero() {
        assert_eval_error!(
            EvalError::CalculationError(CalculationError::ZeroDivision),
            Token::Operator(Operator::Div),
            Token::Number(Number::Int(0)),
            Token::Number(Number::Int(1))
        );
        assert_eval_error!(
            EvalError::CalculationError(CalculationError::ZeroDivision),
            Token::Operator(Operator::Div),
            Token::Number(Number::Float(0.0)),
            Token::Number(Number::Float(1.0))
        );
    }
}
