use colored::Colorize;
use regex::Regex;
use std::str::FromStr;
use std::{env, time::Instant};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UrlEntry {
    pub long_url: String,
    pub short_url: String,
    pub expiration_date: Option<Instant>,
    pub creation_date: Instant,
    pub author: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UrlEntryRequest {
    pub long_url: Option<String>,
    pub short_url: Option<String>,
    pub expiration_date: Option<Instant>,
    pub author: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryParseError {
    message: String,
}

impl UrlEntry {
    pub fn new(long_url: &str, short_url: &str) -> Self {
        UrlEntry {
            long_url: long_url.to_string(),
            short_url: short_url.to_string(),
            expiration_date: None,
            creation_date: Instant::now(),
            author: env::var("USER").unwrap_or("SYSTEM".to_string()),
        }
    }

    pub fn merge_with(&mut self, entry: &UrlEntryRequest) {
        self.long_url = entry.long_url.clone().unwrap_or(self.long_url.clone());
        self.short_url = self.short_url.clone();
        self.creation_date = self.creation_date.clone();
        self.expiration_date = entry.expiration_date.or(self.expiration_date);
        self.author = self.author.clone();
    }

    pub fn has_valid_url(&self) -> bool {
        lazy_static! {
            //A-Z, a-z, 0-9, -, ., _, ~, :, /, ?, #, [, ], @, !, $, &, ', (, ), *, +, ,, ;, %, and =
            static ref URL_EXPRESSION: Regex = Regex::new(r"^[A-Za-z0-9]+$").unwrap();
        }
        URL_EXPRESSION.is_match(&self.long_url)
    }

    pub fn assoc_with(&self, entry: &UrlEntryRequest) -> Self {
        let long_url = entry.long_url.clone().unwrap_or(self.long_url.clone());
        let short_url = self.short_url.clone();
        let creation_date = self.creation_date;
        let expiration_date = entry.expiration_date.or(self.expiration_date);
        let author = entry.author.clone();

        UrlEntry {
            long_url,
            short_url,
            expiration_date,
            creation_date,
            author,
        }
    }

    pub fn to_file_string(&self) -> String {
        format!(
            "{}:{},{:?},{:?},{}",
            self.short_url, self.long_url, self.expiration_date, self.creation_date, self.author
        )
    }
}

impl std::fmt::Display for UrlEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let long_url = "Long URL: ".truecolor(135, 135, 135);
        let short_url = "Short URL: ".truecolor(135, 135, 135);
        let expiration_date = "Expiration Date: ".truecolor(135, 135, 135);
        let creation_date = "Creation Date: ".truecolor(135, 135, 135);
        let author = "Author: ".truecolor(135, 135, 135);
        write!(
            f,
            "{}{}\n{}{}\n{}{:?}\n{}{:?}\n{}{}\n",
            long_url,
            self.long_url,
            short_url,
            self.short_url,
            expiration_date,
            self.expiration_date,
            creation_date,
            self.creation_date,
            author,
            self.author
        )
    }
}

impl FromStr for UrlEntry {
    type Err = EntryParseError;

    /// Takes a `UrlEntry` of the form:
    /// ```
    /// <short_url>:<long-url>,<expiration_date>,<creation_date>,<author>
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref URL_ENTRY_RE: Regex =
                Regex::new(r"^(?P<short>[a-zA-Z0-9]+):(?P<long>\w+),(?P<expiration>.*),(?P<creation>.*),(?P<author>\w+)$").unwrap();
        }

        if let Some(captures) = URL_ENTRY_RE.captures(s) {
            let short_url = captures
                .name("short")
                .expect("Short URL exists")
                .as_str()
                .to_string();

            let long_url = captures
                .name("long")
                .expect("Long URL exists")
                .as_str()
                .to_string();

            let expiration_date = captures
                .name("expiration")
                .expect("expiration exists")
                .as_str()
                .to_string();

            let creation_date = captures
                .name("creation")
                .expect("creation exists")
                .as_str()
                .to_string();

            let author = captures
                .name("author")
                .expect("author exists")
                .as_str()
                .to_string();

            Ok(UrlEntry {
                short_url,
                long_url,
                expiration_date: None,
                creation_date: Instant::now(),
                author,
            })
        } else {
            Err(EntryParseError {
                message: "AHHH".to_string(),
            })
        }
    }
}
