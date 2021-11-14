//! Abstraction to perform asynchronous calculations & queries without blocking UI
//!
//! This opens a `crossbeam` `channel` to communicate with a backend.
//! Each backend operation is send and retrieved in a loop on a thread.
//! This allows sending operations into `Link` and retrieving the contents
//! asynchronously without blocking the UI.

use std::convert::TryInto;

use crossbeam_channel::{unbounded, Receiver, Sender};
use eyre::Result;

use crate::database::{
    query::Query,
    query_result::{QueryResult, QueryRow},
    Database,
};
use crate::types::Config;

use super::types::Segmentation;

#[derive(Debug)]
pub enum Response<Context: Send + 'static> {
    Grouped(Query, Context, Segmentation),
    Normal(Query, Context, Vec<QueryRow>),
}

pub(super) type InputSender<Context> = Sender<(Query, Context)>;
pub(super) type OutputReciever<Context> = Receiver<Result<Response<Context>>>;

pub(super) struct Link<Context: Send + 'static> {
    pub input_sender: InputSender<Context>,
    pub output_receiver: OutputReciever<Context>,
    // We need to account for the brief moment where the processing channel is empty
    // but we're applying the results. If there is a UI update in this window,
    // the UI will not update again after the changes were applied because an empty
    // channel indicates completed processing.
    // There's also a delay between a request taken out of the input channel and being
    // put into the output channel. In order to account for all of this, we emploty a
    // request counter to know how many requests are currently in the pipeline
    request_counter: usize,
}

impl<Context: Send + Sync + 'static> Link<Context> {
    pub fn request(&mut self, query: &Query, context: Context) -> Result<()> {
        self.request_counter += 1;
        self.input_sender.send((query.clone(), context))?;
        Ok(())
    }

    pub fn receive(&mut self) -> Result<Option<Response<Context>>> {
        match self.output_receiver.try_recv() {
            // We received something
            Ok(Ok(response)) => {
                // Only subtract if we successfuly received a value
                self.request_counter -= 1;
                Ok(Some(response))
            }
            // We received nothing
            Err(_) => Ok(None),
            // There was an error, we forward it
            Ok(Err(e)) => Err(e),
        }
    }

    pub fn is_processing(&self) -> bool {
        self.request_counter > 0
    }
}

pub(super) fn run<Context: Send + Sync + 'static>(config: &Config) -> Result<Link<Context>> {
    // Create a new database connection, just for reading
    let database = Database::new(&config.database_path)?;
    let (input_sender, input_receiver) = unbounded();
    let (output_sender, output_receiver) = unbounded();
    let _ = std::thread::spawn(move || inner_loop(database, input_receiver, output_sender));
    Ok(Link {
        input_sender,
        output_receiver,
        request_counter: 0,
    })
}

fn inner_loop<Context: Send + Sync + 'static>(
    database: Database,
    input_receiver: Receiver<(Query, Context)>,
    output_sender: Sender<Result<Response<Context>>>,
) -> Result<()> {
    loop {
        let (query, context) = input_receiver.recv()?;
        let result = database.query(&query)?;
        let response = match query {
            Query::Grouped { .. } => {
                let segmentations = calculate_segmentations(&result)?;
                Response::Grouped(query, context, segmentations)
            }
            Query::Normal { .. } => {
                let converted = calculate_rows(&result)?;
                Response::Normal(query, context, converted)
            }
        };
        output_sender.send(Ok(response))?;
    }
}

fn calculate_segmentations(result: &[QueryResult]) -> Result<Segmentation> {
    let mut segmentations = Vec::new();
    for r in result.iter() {
        let segmentation = r.try_into()?;
        segmentations.push(segmentation);
    }

    Ok(Segmentation::new(segmentations))
}

fn calculate_rows(result: &[QueryResult]) -> Result<Vec<QueryRow>> {
    Ok(result
        .iter()
        .map(|r| {
            let values = match r {
                QueryResult::Normal(values) => values,
                _ => {
                    panic!("Invalid result type, expected `Normal`")
                }
            };
            values.clone()
        })
        .collect())
}
