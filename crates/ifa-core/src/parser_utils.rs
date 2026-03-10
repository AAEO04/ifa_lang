// use crate::ast::Span;
use crate::error::{IfaError, IfaResult};
use crate::parser::Rule;

/// Safely get the next pair from an iterator, or return a parse error
pub fn safe_next<'a>(
    iter: &mut pest::iterators::Pairs<'a, Rule>,
    context: &str,
) -> IfaResult<pest::iterators::Pair<'a, Rule>> {
    iter.next()
        .ok_or_else(|| IfaError::Parse(format!("Unexpected end of input in {}", context)))
}
