pub mod lexer;
pub mod token;

pub use lexer::Lexer;
pub use token::{LexError, Span, SpannedTokenKind, Token, TokenKind, TokenizationResult};

#[cfg(test)]
mod tests;
