use eyre::Result;

use std::{
    io::{stdout, Write},
    thread::sleep,
    time::Duration,
};

use gmaildb::*;

fn main() -> Result<()> {
    setup_tracing();

    let config = make_config();

    println!("Collecting Mails...");
    let emails = importer::filesystem::read_emails(&config)?;

    println!("Begin Parsing Mails...");
    let (receiver, handle) = crate::importer::parse::emails::parse_emails(&config, emails)?;

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
            use importer::parse::emails::ParseMessage;
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
    tracing::trace!("Exit Program");
    Ok(())
}
