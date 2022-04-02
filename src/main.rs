#[macro_use]
extern crate lazy_static;
use clap::{Parser, Subcommand};
use persistence::Persistence;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::Instant;
mod persistence;
mod url_entry;
use crate::url_entry::{UrlEntry, UrlEntryRequest};
use colored::Colorize;

const ALPHABET: &'static [char] = &[
    '0', '1', '2', '3', '4', '5', '6', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
    'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C',
    'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V',
    'W', 'X', 'Y', 'Z',
];

fn create_hash(long_url: &str) -> Option<u64> {
    let mut hasher = DefaultHasher::new();
    long_url.hash(&mut hasher);
    Some(hasher.finish())
}

fn encode_hash(mut hash: u64) -> String {
    let mut encoded = String::new();

    while hash > 0 {
        encoded.push(ALPHABET[(hash % 62) as usize]);
        hash /= 62;
    }

    encoded
}

/// Adds a URL to the configured persistence layer
///
/// If the entry was successfully created it is also returned.
/// If the entry wasn't successfully created, `None` is returned
///
/// If a collision occurs, the method will `panic`
///
/// # Example
/// ```
/// let mut url_map: HashMap<String, UrlEntry> = HashMap::new();
/// let entry: UrlEntry = add_url("https://www.google.com", url_map);
/// ```
fn add_url<T>(long_url: &str, map: &mut T) -> Option<UrlEntry>
where
    T: Persistence,
{
    let hash = create_hash(long_url)?;
    let short_url = encode_hash(hash);

    if map.contains_key(&short_url) {
        panic!("There was a collision");
    } else {
        let entry = UrlEntry::new(long_url, &short_url);
        map.insert(short_url.clone(), entry.clone());
        return Some(entry);
    }
}

/// Adds a `UrlEntry` to the configured persistence layer
///
/// If the entry was successfully created it is also returned.
/// If the entry wasn't successfully created, `None` is returned
///
/// If a collision occurs, the method will `panic`
fn add_entry<'a, T: 'a>(entry: UrlEntry, map: &'a mut T) -> Option<&'a UrlEntry>
where
    T: Persistence,
{
    let short_url = encode_hash(create_hash(&entry.long_url)?);
    map.insert(
        short_url.clone(),
        UrlEntry {
            long_url: entry.long_url.clone(),
            short_url,
            expiration_date: entry.expiration_date,
            creation_date: Instant::now(),
            author: entry.author,
        },
    )
}

/// The provided `UrlEntryRequest` will be merged with the `UrlEntry` associated with the given `short_url`
///
/// If there is no `UrlEntry` associated with the given encoding, `None` will be returned
///
/// If the entry was successfully merged, the updated version will be returned
fn edit_entry<'a, T>(
    short_url: &str,
    goal_entry: UrlEntryRequest,
    persistence: &'a T,
) -> Option<UrlEntry>
where
    T: Persistence,
{
    let mut entry = persistence.get(short_url)?;

    entry.merge_with(&goal_entry);

    Some(entry.clone())
}

/// Adds a `UrlEntry` to the configured persistence layer
///
/// If the entry was successfully created it is also returned.
/// If the entry wasn't successfully created, `None` is returned
///
/// If a collision occurs, the method will `panic`
fn get_url<T>(short_url: &str, url_map: &T) -> Option<UrlEntry>
where
    T: Persistence,
{
    Some(url_map.get(short_url)?.to_owned())
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        long_url: String,
    },
    View {
        short_url: String,
        /// Show full entry information
        #[clap(short, long)]
        long: bool,
    },
    Edit {
        short_url: String,
        //entry: UrlEntryRequest,
    },
    Remove {
        short_url: String,
    },
}

fn main() {
    let args = Cli::parse();

    let mut persistence = persistence::File::new();

    let added = "ADDED:\n".green().bold();

    match &args.command {
        Commands::Add { long_url } => {
            let a = add_url(long_url, &mut persistence).unwrap();

            let long_url = "\tLong URL: ".truecolor(135, 135, 135);
            let short_url = "\tShort URL: ".truecolor(135, 135, 135);
            let expiration_date = "\tExpiration Date: ".truecolor(135, 135, 135);
            let creation_date = "\tCreation Date: ".truecolor(135, 135, 135);
            let author = "\tAuthor: ".truecolor(135, 135, 135);
            println!(
                "{}\n{}{}\n{}{}\n{}{:?}\n{}{:?}\n{}{}\n",
                added,
                long_url,
                a.long_url,
                short_url,
                a.short_url,
                expiration_date,
                a.expiration_date,
                creation_date,
                a.creation_date,
                author,
                a.author
            );
        }
        Commands::View { short_url, long } => {
            if let Some(entry) = persistence.get(short_url) {
                if *long {
                    println!("{} => {:?}", short_url.green(), entry);
                } else {
                    println!("{} => {:?}", short_url.green(), entry.long_url);
                }
            } else {
                println!("{}", "Not Found".red().bold());
            }
        }
        Commands::Edit { short_url } => {
            if let Some(entry) = persistence.get(short_url) {
                println!("{} => {:?}", short_url.green(), entry);
            } else {
                println!("{}", "Not Found".red().bold());
            }
        }
        Commands::Remove { short_url } => {
            println!("Removing {:?}", short_url);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_same_output_hash() {
        assert_eq!(create_hash("blob"), create_hash("blob"));
    }

    #[test]
    fn hash_doesnt_trivially_collide() {
        assert_ne!(create_hash("blob"), create_hash("bolb"));
    }

    #[test]
    fn doesnt_accept_bad_urls() {
        let entry = UrlEntry::new("  asdf )(*)", "");
        assert!(!entry.has_valid_url());
    }
}
