use eyre::Result;

use std::{
    io::{stdout, Write},
    thread::sleep,
    time::Duration,
};

use postsack::{
    self,
    importer::{Adapter, State},
    types::FormatType,
};

fn main() -> Result<()> {
    postsack::setup_tracing();

    let config = postsack::make_config();

    let adapter = postsack::importer::Adapter::new();

    // Could not figure out how to build this properly
    // with dynamic dispatch. (to abstract away the match)
    // Will try again when I'm online.
    let handle = match config.format {
        FormatType::AppleMail => {
            let importer = postsack::importer::applemail_importer(config);
            adapter.process(importer)?
        }
        FormatType::GmailVault => {
            let importer = postsack::importer::gmail_importer(config);
            adapter.process(importer)?
        }
        FormatType::Mbox => {
            let importer = postsack::importer::mbox_importer(config);
            adapter.process(importer)?
        }
    };

    let mut stdout = stdout();

    loop {
        match handle_adapter(&adapter) {
            Ok(true) => break,
            Ok(false) => (),
            Err(e) => {
                println!("Execution Error:\n{:?}", &e);
                panic!();
            }
        }
        stdout.flush().unwrap();
    }

    match handle.join() {
        Err(e) => println!("Error: {:?}", e),
        Ok(Err(e)) => println!("Error: {:?}", e),
        _ => (),
    }
    println!("\rDone");

    Ok(())
}

fn handle_adapter(adapter: &Adapter) -> Result<bool> {
    let State {
        done, finishing, ..
    } = adapter.finished()?;
    if done {
        return Ok(true);
    }
    if finishing {
        print!("\rFinishing up...");
    } else {
        let write = adapter.write_count()?;
        if write.count > 0 {
            print!("\rWriting emails to DB {}/{}...", write.count, write.total);
        } else {
            let read = adapter.read_count()?;
            print!(
                "\rReading Emails {}%...",
                (read.count as f32 / read.total as f32) * 100.0
            );
        }
    }
    sleep(Duration::from_millis(50));
    Ok(false)
}
