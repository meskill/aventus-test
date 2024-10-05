mod eval;
mod parse;
mod tokens;

use crate::tokens::TokenIterator;

use self::{
    eval::{EvalError, Evaluator},
    parse::{ExprParser, ParserError},
};

#[derive(Debug)]
pub enum ExprError {
    ParserError(ParserError),
    EvalError(EvalError),
}

impl std::fmt::Display for ExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParserError(err) => write!(f, "{err}"),
            Self::EvalError(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ExprError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParserError(err) => Some(err),
            Self::EvalError(err) => Some(err),
        }
    }
}

impl From<ParserError> for ExprError {
    fn from(error: ParserError) -> Self {
        Self::ParserError(error)
    }
}

impl From<EvalError> for ExprError {
    fn from(error: EvalError) -> Self {
        Self::EvalError(error)
    }
}

pub type Result<T> = std::result::Result<T, ExprError>;

/// Evaluates the expression from string with default settings
pub fn eval(expr: &str) -> Result<f64> {
    let mut tokens = TokenIterator::from(expr);
    let mut parser = ExprParser::new();
    let mut parsed = parser.parse(&mut tokens)?;
    let evaluator = Evaluator::new();

    Ok(evaluator.eval(&mut parsed)?)
}
