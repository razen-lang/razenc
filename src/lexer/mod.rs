pub mod token;
pub mod lexer;

pub use token::{Token, TokenKind, SpannedTokenKind, LexError, TokenizationResult, Span};
pub use lexer::Lexer;

#[cfg(test)]
mod tests;
