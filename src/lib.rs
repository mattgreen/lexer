mod lexicon;
mod lexer;
mod nfa;

pub use crate::lexicon::{Lexicon, LexiconBuilder, Error as LexiconBuilderError};
pub use crate::lexer::{Lexer, Next, Error, Position};

#[cfg(test)]
mod tests {
    use crate::*;

    fn simple_lexicon() -> Lexicon {
        LexiconBuilder::new()
            .ignore_chars(" ")
            .rule(0, r"[a-zA-Z]+").unwrap()
            .rule(1, r"[0-9]+").unwrap()
            .build()
    }

    #[test]
    fn empty() {
        let lexicon = simple_lexicon();
        let mut lexer = Lexer::new(&lexicon, "");

        assert_eq!(lexer.next(), Next::End);
        assert_eq!(lexer.next(), Next::End);
    }

    #[test]
    fn invalid() {
        let lexicon = simple_lexicon();
        let mut lexer = Lexer::new(&lexicon, "a b 1 -     ");

        assert_eq!(lexer.next(), Next::Token(0, "a", Position::new(1, 1)));
        assert_eq!(lexer.next(), Next::Token(0, "b", Position::new(1, 3)));
        assert_eq!(lexer.next(), Next::Token(1, "1", Position::new(1, 5)));
        assert_eq!(lexer.next(), Next::Error(Error::UnexpectedChar("-"), Position::new(1, 7)));
        assert_eq!(lexer.next(), Next::End);
    }

    #[test]
    fn whitespace() {
        let lexicon = simple_lexicon();
        let mut lexer = Lexer::new(&lexicon, "       ");

        assert_eq!(lexer.next(), Next::End);
    }

    #[test]
    fn words() {
        let lexicon = simple_lexicon();
        let mut lexer = Lexer::new(&lexicon, "   abc AAaa 123   ");

        assert_eq!(lexer.next(), Next::Token(0, "abc", Position::new(1, 4)));
        assert_eq!(lexer.next(), Next::Token(0, "AAaa", Position::new(1, 8)));
        assert_eq!(lexer.next(), Next::Token(1, "123", Position::new(1, 13)));
        assert_eq!(lexer.next(), Next::End);
    }
}
