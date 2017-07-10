use std::collections::HashMap;
use std::ops::{Deref, Index};
use std::sync::Arc;
use regex::Regex;
use typemap::Key;


pub struct RegexPattern {
    pattern: Regex,
    names: Arc<HashMap<String, usize>>,
}

impl Deref for RegexPattern {
    type Target = Regex;
    fn deref(&self) -> &Self::Target {
        &self.pattern
    }
}

impl From<Regex> for RegexPattern {
    fn from(pattern: Regex) -> Self {
        let names = pattern
            .capture_names()
            .enumerate()
            .filter_map(|(i, name)| name.map(|name| (name.to_owned(), i)))
            .collect();
        RegexPattern {
            pattern,
            names: Arc::new(names),
        }
    }
}

impl RegexPattern {
    pub fn owned_captures(&self, text: &str) -> Option<OwnedCaptures> {
        self.pattern.captures(text).map(|caps| {
            let matches = caps.iter()
                .map(|cap| cap.map(|m| (m.start(), m.end())))
                .collect();

            OwnedCaptures {
                text: text.to_owned(),
                matches,
                names: self.names.clone(),
            }
        })
    }
}


#[derive(Debug, Clone)]
pub struct OwnedCaptures {
    text: String,
    matches: Vec<Option<(usize, usize)>>,
    names: Arc<HashMap<String, usize>>,
}

impl OwnedCaptures {
    pub fn get(&self, i: usize) -> Option<&str> {
        self.matches.get(i).and_then(|m| {
            m.map(|(start, end)| &self.text[start..end])
        })
    }

    pub fn name(&self, name: &str) -> Option<&str> {
        self.names.get(name).and_then(|&i| self.get(i))
    }
}

impl Index<usize> for OwnedCaptures {
    type Output = str;
    fn index(&self, i: usize) -> &str {
        self.get(i).unwrap()
    }
}

impl<'i> Index<&'i str> for OwnedCaptures {
    type Output = str;
    fn index<'a>(&'a self, name: &'i str) -> &'a str {
        self.name(name).unwrap()
    }
}

impl Key for OwnedCaptures {
    type Value = Self;
}
