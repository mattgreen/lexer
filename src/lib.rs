mod lexer;
mod lexicon;
mod nfa;

pub use crate::lexer::{Error, Lexer, Next, Position};
pub use crate::lexicon::{Error as LexiconBuilderError, Lexicon, LexiconBuilder};

#[cfg(test)]
mod tests {
    use crate::*;

    fn simple_lexicon() -> Lexicon {
        LexiconBuilder::new()
            .ignore_chars(" ")
            .pattern(0, r"[a-zA-Z]+")
            .pattern(1, r"[0-9]+")
            .literal(2, "if")
            .build()
            .unwrap()
    }

    #[test]
    fn empty() {
        let lexicon = simple_lexicon();
        let mut lexer = Lexer::new(&lexicon, "");

        assert_eq!(lexer.next(), None);
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn invalid() {
        let lexicon = simple_lexicon();
        let mut lexer = Lexer::new(&lexicon, "a b 1 -     ");

        assert_eq!(lexer.next(), Some(Next::Token(0, "a", Position::new(1, 1))));
        assert_eq!(lexer.next(), Some(Next::Token(0, "b", Position::new(1, 3))));
        assert_eq!(lexer.next(), Some(Next::Token(1, "1", Position::new(1, 5))));
        assert_eq!(
            lexer.next(),
            Some(Next::Error(Error::UnexpectedChar("-"), Position::new(1, 7)))
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn whitespace() {
        let lexicon = simple_lexicon();
        let mut lexer = Lexer::new(&lexicon, "       ");

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn words() {
        let lexicon = simple_lexicon();
        let mut lexer = Lexer::new(&lexicon, "   abc AAaa 123   ");

        assert_eq!(lexer.next(), Some(Next::Token(0, "abc", Position::new(1, 4))));
        assert_eq!(lexer.next(), Some(Next::Token(0, "AAaa", Position::new(1, 8))));
        assert_eq!(lexer.next(), Some(Next::Token(1, "123", Position::new(1, 13))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn literals() {
        let lexicon = simple_lexicon();
        let mut lexer = Lexer::new(&lexicon, "   abc if iffy 123   ");

        assert_eq!(lexer.next(), Some(Next::Token(0, "abc", Position::new(1, 4))));
        assert_eq!(lexer.next(), Some(Next::Token(2, "if", Position::new(1, 8))));
        assert_eq!(lexer.next(), Some(Next::Token(0, "iffy", Position::new(1, 11))));
        assert_eq!(lexer.next(), Some(Next::Token(1, "123", Position::new(1, 16))));
        assert_eq!(lexer.next(), None);
    }
}
