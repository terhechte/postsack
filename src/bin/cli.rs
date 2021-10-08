use eyre::Result;

use std::{
    io::{stdout, Write},
    thread::sleep,
    time::Duration,
};

use gmaildb::{
    self,
    importer::{Adapter, State},
};

fn main() -> Result<()> {
    gmaildb::setup_tracing();

    let config = gmaildb::make_config();
    let importer = gmaildb::importer::gmail_importer(&config);

    let adapter = gmaildb::importer::Adapter::new();
    let handle = adapter.process(importer)?;

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
    let State { done, finishing } = adapter.finished()?;
    if done {
        return Ok(true);
    }
    if finishing {
        print!("\rFinishing up...");
    } else {
        let write = adapter.write_count()?;
        if write.count > 0 {
            print!("\rWriting to DB {}/{}...", write.count, write.total);
        } else {
            let read = adapter.read_count()?;
            print!(
                "\rReading Emails {}%...",
                ((read.count as f32 / read.total as f32) * 100.0) as usize
            );
        }
    }
    sleep(Duration::from_millis(50));
    Ok(false)
}
