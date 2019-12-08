use regex::Regex;

pub struct Lexicon<'input, T> {
    pub(crate) rules: Vec<Rule<'input, T>>,
}

#[derive(Default)]
pub struct LexiconBuilder<'input, T> {
    rules: Vec<Rule<'input, T>>,
}

pub enum Rule<'input, T> {
    Ignore(Regex),
    Token(Regex, Handler<'input, T>),
}

pub type Handler<'input, T> = fn(&'input str) -> T;

#[derive(Debug)]
pub enum Error {
    Regex(regex::Error),
}

impl<'input, T> LexiconBuilder<'input, T> {
    pub fn new() -> Self {
        Self { rules: vec![] }
    }

    pub fn build(self) -> Lexicon<'input, T> {
        Lexicon { rules: self.rules }
    }

    pub fn ignore(mut self, pattern: &str) -> Result<Self, Error> {
        let anchored = Self::anchor(pattern);
        let regex = Regex::new(&anchored).map_err(Error::Regex)?;

        self.rules.push(Rule::Ignore(regex));

        Ok(self)
    }

    pub fn token(mut self, pattern: &str, handler: Handler<'input, T>) -> Result<Self, Error> {
        let anchored = Self::anchor(pattern);
        let regex = Regex::new(&anchored).map_err(Error::Regex)?;

        self.rules.push(Rule::Token(regex, handler));

        Ok(self)
    }

    fn anchor(pattern: &str) -> String {
        let mut anchored = pattern.to_owned();
        if !anchored.starts_with('^') && !anchored.starts_with("\\A") {
            anchored.insert(0, '^');
        }

        anchored
    }
}

impl<'input, T> Rule<'input, T> {
    pub(crate) fn match_len(&self, input: &str) -> Option<usize> {
        let regex = match self {
            Self::Ignore(r) => r,
            Self::Token(r, _) => r,
        };

        regex.find(input).map(|m| m.end() - m.start())
    }
}