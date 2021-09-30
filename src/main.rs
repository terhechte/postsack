use core::num;
use eyre::{bail, Result};
use rayon::prelude::*;
use std::io::prelude::*;
use std::thread::JoinHandle;
use std::{io, path::PathBuf};
use thiserror;
use tracing_subscriber::EnvFilter;

use crossbeam_channel;
use std::path::Path;
use std::sync::{Arc, Mutex};

use std::{
    io::{stdout, Write},
    thread::sleep,
    time::Duration,
};

use crate::database::Database;

mod database;
mod filesystem;
mod parse;
mod types;

#[derive(Debug, thiserror::Error)]
enum GmailDBError {
    #[error("Missing folder argument")]
    MissingFolder,
}

fn main() -> Result<()> {
    setup();
    let arguments: Vec<String> = std::env::args().collect();
    let folder = arguments
        .get(1)
        .unwrap_or_else(|| panic!("Missing folder path argument"));
    let database = arguments
        .get(2)
        .unwrap_or_else(|| panic!("Missing database path argument"));
    let config = crate::types::Config::new(database, folder);

    println!("Collecting Mails...");
    let emails = filesystem::read_emails(&config)?;

    println!("Begin Parsing Mails...");
    let (receiver, handle) = crate::parse::emails::parse_emails(&config, emails)?;

    let mut stdout = stdout();

    let mut total: Option<usize> = None;
    let mut counter = 0;
    let mut done = false;

    'outer: while done == false {
        for entry in receiver.try_iter() {
            let message = match entry {
                Ok(n) => n,
                Err(e) => {
                    println!("Processing Error: {:?}", &e);
                    break 'outer;
                }
            };
            use parse::emails::ParseMessage;
            match message {
                ParseMessage::Done => done = true,
                ParseMessage::Total(n) => total = Some(n),
                ParseMessage::ParsedOne => counter += 1,
            };
        }

        if let Some(total) = total {
            print!("\rProcessing {}/{}...", counter, total);
        }

        stdout.flush().unwrap();
        sleep(Duration::from_millis(20));
    }
    let result = handle.join().map_err(|op| eyre::eyre!("{:?}", &op))??;

    println!(
        "Read: {}, Processed: {}, Inserted: {}",
        total.unwrap_or_default(),
        counter,
        result
    );

    println!();
    //process_email(&folder)?;
    tracing::trace!("Exit Program");
    Ok(())
}

fn setup() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
