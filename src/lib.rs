mod lexicon;
mod lexer;

pub use crate::lexicon::{Lexicon, LexiconBuilder, Error as LexiconBuilderError};
pub use crate::lexer::{Lexer, Next, Error};

#[cfg(test)]
mod tests {
    use crate::*;

    #[derive(Debug, PartialEq)]
    enum Token<'input> {
        Int(&'input str),
        Name(&'input str),
    }

    fn simple_lexicon<'input>() -> Lexicon<'input, Token<'input>> {
        LexiconBuilder::new()
            .ignore_chars(" ")
            .token(r"[a-zA-Z]+", Token::Name).unwrap()
            .token(r"[0-9]+", Token::Int).unwrap()
            .build()
    }

    #[test]
    fn empty() {
        let mut lexer = Lexer::new(simple_lexicon(), "");

        assert_eq!(lexer.next(), Next::End);
        assert_eq!(lexer.next(), Next::End);
    }

    #[test]
    fn invalid() {
        let mut lexer = Lexer::new(simple_lexicon(), "a b 1 -     ");

        assert_eq!(lexer.next(), Next::Token(Token::Name("a")));
        assert_eq!(lexer.next(), Next::Token(Token::Name("b")));
        assert_eq!(lexer.next(), Next::Token(Token::Int("1")));
        assert_eq!(lexer.next(), Next::Error(Error::UnexpectedChar('-')));
        assert_eq!(lexer.next(), Next::End);
    }

    #[test]
    fn whitespace() {
        let mut lexer = Lexer::new(simple_lexicon(), "       ");

        assert_eq!(lexer.next(), Next::End);
    }

    #[test]
    fn words() {
        let mut lexer = Lexer::new(simple_lexicon(), "   abc AAaa 123   ");

        assert_eq!(lexer.next(), Next::Token(Token::Name("abc")));
        assert_eq!(lexer.next(), Next::Token(Token::Name("AAaa")));
        assert_eq!(lexer.next(), Next::Token(Token::Int("123")));
        assert_eq!(lexer.next(), Next::End);
    }
}
