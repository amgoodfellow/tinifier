use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::{collections::HashMap, path::Path};

use crate::url_entry::UrlEntry;

pub trait Persistence {
    fn insert<S>(&mut self, short_url: S, entry: UrlEntry) -> Option<&UrlEntry>
    where
        S: Into<String> + Clone;
    fn get<S>(&self, short_url: S) -> Option<UrlEntry>
    where
        S: Into<String> + Clone;
    fn remove<S>(&mut self, short_url: S) -> Option<UrlEntry>
    where
        S: Into<String> + Clone;
    fn contains_key<S>(&self, short_url: S) -> bool
    where
        S: Into<String> + Clone;
}

pub struct InMemory {
    map: HashMap<String, UrlEntry>,
}

impl Persistence for InMemory {
    fn insert<S>(&mut self, short_url: S, entry: UrlEntry) -> Option<&UrlEntry>
    where
        S: Into<String> + Clone,
    {
        self.map.insert(short_url.clone().into(), entry);
        self.map.get(&short_url.into())
    }

    fn get<S>(&self, short_url: S) -> Option<UrlEntry>
    where
        S: Into<String> + Clone,
    {
        Some(self.map.get(&short_url.into())?.to_owned())
    }

    fn remove<S>(&mut self, short_url: S) -> Option<UrlEntry>
    where
        S: Into<String> + Clone,
    {
        self.map.remove(&short_url.into())
    }

    fn contains_key<S>(&self, short_url: S) -> bool
    where
        S: Into<String> + Clone,
    {
        true
    }
}

pub struct File<'a> {
    map: HashMap<String, UrlEntry>,
    file_location: &'a Path,
}

impl File<'_> {
    pub fn new() -> Self {
        if let Some(file) = OpenOptions::new().read(true).open("/tmp/tinifier").ok() {
            let lines = BufReader::new(file)
                .lines()
                .filter_map(|line| line.ok())
                .filter_map(|line| {
                    if let Some(entry) = line.parse::<UrlEntry>().ok() {
                        return Some((entry.short_url.clone(), entry));
                    } else {
                        return None;
                    }
                })
                .collect::<HashMap<String, UrlEntry>>();
            return File {
                map: lines,
                file_location: Path::new("/tmp/tinifier"),
            };
        }
        File {
            map: HashMap::new(),
            file_location: Path::new("/tmp/tinifier"),
        }
    }
}

impl Persistence for File<'_> {
    fn insert<S>(&mut self, short_url: S, entry: UrlEntry) -> Option<&UrlEntry>
    where
        S: Into<String> + Clone,
    {
        // Insert into cache
        self.map.insert(short_url.clone().into(), entry.clone());
        // Write to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.file_location)
            .expect("file cannot be opened");

        // If there's an error writing, remove from hashmap
        if let Err(e) = writeln!(file, "{}", &entry.to_file_string()) {
            self.map.remove(&short_url.clone().into());
            eprintln!("Couldn't write to file: {}", e);
        }

        // Return a reference to the entry we created
        self.map.get(&short_url.into())
    }

    fn get<S>(&self, short_url: S) -> Option<UrlEntry>
    where
        S: Into<String> + Clone,
    {
        // Get from the cache
        Some(self.map.get(&short_url.into())?.to_owned())
    }

    fn remove<S>(&mut self, short_url: S) -> Option<UrlEntry>
    where
        S: Into<String> + Clone,
    {
        // Open the persistence file
        let file = OpenOptions::new()
            .read(true)
            .open(self.file_location)
            .expect("file doesn't exist");

        // Read from the file, filtering out any lines containing the short_url
        // Then collect the result into a new string (AH BAD)
        let lines = BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| {
                if line.contains(&short_url.clone().into()) {
                    None
                } else {
                    Some(line)
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        // Write the updated string to the file
        std::fs::write(self.file_location, lines).expect("Writing failed");

        // Remove the entry from the cache, returning it
        self.map.remove(&short_url.into())
    }

    fn contains_key<S>(&self, short_url: S) -> bool
    where
        S: Into<String> + Clone,
    {
        false
    }
}
