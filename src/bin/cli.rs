use eyre::Result;

use std::{
    io::{stdout, Write},
    thread::sleep,
    time::Duration,
};

use gmaildb::{
    self,
    importer::{Progress, State},
};

fn main() -> Result<()> {
    gmaildb::setup_tracing();

    let config = gmaildb::make_config();
    let importer = gmaildb::importer::gmail_importer(&config);

    let adapter = gmaildb::importer::Adapter::new();
    let handle = adapter.process(importer)?;

    let mut stdout = stdout();

    loop {
        match adapter.finished() {
            Ok(State {
                finishing: true, ..
            }) => {
                println!("Finishing import...");
            }
            Ok(State { done: true, .. }) => {
                break;
            }
            _ => (),
        };
        match adapter.read_count() {
            Ok(Progress { count, total }) => {
                print!("\rReading {}/{}...", count, total);
            }
            _ => (),
        };
        match adapter.write_count() {
            Ok(Progress { count, total }) => {
                print!("\rWriting to DB {}/{}...", count, total);
            }
            _ => (),
        };
        stdout.flush().unwrap();
        sleep(Duration::from_millis(50));
    }

    match handle.join() {
        Err(e) => println!("Error: {:?}", e),
        Ok(Err(e)) => println!("Error: {:?}", e),
        _ => (),
    }

    Ok(())
}
