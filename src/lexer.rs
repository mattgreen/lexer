use crate::Lexicon;

use crate::lexicon::Rule;

pub struct Lexer<'input, T> {
    lexicon: Lexicon<'input, T>,
    input: &'input str,
    offset: usize,
}

#[derive(Debug, PartialEq)]
pub enum Next<T> {
    Token(T),
    End,
    Error(Error),
}

#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedChar(char),
}

impl<'input, T> Lexer<'input, T> {
    pub fn new(lexicon: Lexicon<'input, T>, input: &'input str) -> Self {
        Self {
            lexicon,
            input,
            offset: 0,
        }
    }

    fn eof(&self) -> bool {
        self.offset >= self.input.len()
    }

    pub fn next(&mut self) -> Next<T> {
        while !self.eof() {
            let input = &self.input[self.offset..];

            let c = input.chars().nth(0).unwrap();
            if self.lexicon.ignore_chars.contains(&c) {
                self.offset += 1;
                continue;
            }

            let matching_rule = self.lexicon.rules
                .iter()
                .filter_map(|r| r.match_len(input).map(|len| (r, len)))
                .nth(0);

            if matching_rule.is_none() {
                self.offset += 1;

                return Next::Error(Error::UnexpectedChar(c));
            }

            let (rule, match_len) = matching_rule.unwrap();

            let token = match rule {
                Rule::Ignore(_) => None,
                Rule::Token(_, handler) => {
                    let slice = &input[..match_len];

                    Some(Next::Token(handler(slice)))
                }
            };

            self.offset += match_len;

            if let Some(t) = token {
                return t;
            }
        }

        Next::End
    }
}
