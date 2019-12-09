use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

use crate::Lexicon;

pub struct Lexer<'input> {
    lexicon: Lexicon,
    patterns: Regex,
    input: &'input str,
    offset: usize,
}

#[derive(Debug, PartialEq)]
pub enum Next<'input> {
    Token(usize, &'input str),
    End,
    Error(Error<'input>),
}

#[derive(Debug, PartialEq)]
pub enum Error<'input> {
    UnexpectedChar(&'input str),
}

impl<'input> Lexer<'input> {
    pub fn new(lexicon: Lexicon, input: &'input str) -> Self {
        let pattern = lexicon.rules
            .iter()
            .map(|r| format!("(\\A{})", r.regex.as_str()))
            .collect::<Vec<String>>()
            .join("|");

        let patterns = Regex::new(&pattern).unwrap();

        Self {
            lexicon,
            patterns,
            input,
            offset: 0,
        }
    }

    fn eof(&self) -> bool {
        self.offset >= self.input.len()
    }

    pub fn next(&mut self) -> Next {
        while !self.eof() {
            let input = &self.input[self.offset..];

            let g = input.graphemes(true).nth(0).unwrap();
            if self.lexicon.ignore_chars.contains(g) {
                self.offset += g.len();
                continue;
            }

            let found = self.patterns.captures(input);
            if found.is_none() {
                self.offset += g.len();

                return Next::Error(Error::UnexpectedChar(g));
            }

            let captures = found.unwrap();

            let mut rule = &self.lexicon.rules[0];
            let mut match_len = 0;

            for i in 1..=self.lexicon.rules.len() {
                if let Some(m) = captures.get(i) {
                    let m_len = m.end() - m.start();

                    if m_len > match_len {
                        rule = &self.lexicon.rules[i - 1];
                        match_len = m_len;
                    }
                }
            }

            self.offset += match_len;

            return Next::Token(rule.id, &input[..match_len]);
        }

        Next::End
    }
}
