use regex::{CaptureLocations, Regex};
use unicode_segmentation::UnicodeSegmentation;

use crate::Lexicon;

pub struct Lexer<'input, 'lexicon> {
    input: &'input str,
    offset: usize,
    lexicon: &'lexicon Lexicon,
    patterns: Regex,
    locations: CaptureLocations,
    ignore_chars_ascii: [bool; 256],
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

impl<'input, 'lexicon> Lexer<'input, 'lexicon> {
    pub fn new(lexicon: &'lexicon Lexicon, input: &'input str) -> Self {
        let pattern = lexicon.rules
            .iter()
            .map(|r| format!("(\\A{})", r.pattern()))
            .collect::<Vec<String>>()
            .join("|");

        let mut ignore_chars_ascii: [bool; 256] = [false; 256];
        for s in lexicon.ignore_chars.iter() {
            let b = s.bytes().nth(0).unwrap();
            if b.is_ascii() {
                ignore_chars_ascii[b as usize] = true;
            }
        }

        let patterns = Regex::new(&pattern).unwrap();
        let locations = patterns.capture_locations();

        Self {
            input,
            offset: 0,
            lexicon,
            patterns,
            locations,
            ignore_chars_ascii,
        }
    }

    fn eof(&self) -> bool {
        self.offset >= self.input.len()
    }

    pub fn next(&mut self) -> Next {
        while !self.eof() {
            let input = &self.input[self.offset..];

            let b = input.bytes().nth(0).unwrap();

            if b.is_ascii() {
                if self.ignore_chars_ascii[b as usize] {
                    self.offset += 1;
                    continue;
                }
            }

            let g = input.graphemes(true).nth(0).unwrap();
            if self.lexicon.ignore_chars.contains(g) {
                self.offset += g.len();
                continue;
            }

            let matched = self.patterns.captures_read(&mut self.locations, input);
            if matched.is_none() {
                self.offset += g.len();
                return Next::Error(Error::UnexpectedChar(g));
            }

            let match_len = matched.map(|m| m.end() - m.start()).unwrap();

            let mut rule = &self.lexicon.rules[0];
            for i in 1..=self.lexicon.rules.len() {
                if self.locations.get(i).is_some() {
                    rule = &self.lexicon.rules[i - 1];
                    break;
                }
            }

            self.offset += match_len;

            return Next::Token(rule.id(), &input[..match_len]);
        }

        Next::End
    }
}
