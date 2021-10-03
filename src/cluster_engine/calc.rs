//! Runs a continuous thread to calculate the canvas.
//! Receives as input the current gui app state and size via a channel,
//! Then performs the SQLite query
//! Then performs the calculation to the `TreeMap`
//! And finally uses a channel to submit the result back to the UI
//! Runs its own connection to the SQLite database.

use std::convert::TryInto;
use std::thread::JoinHandle;

use crossbeam_channel::{unbounded, Receiver, Sender};
use eyre::{Report, Result};

use crate::database::{
    query::{Filter, GroupByField, Query},
    query_result::QueryResult,
    Database,
};
use crate::types::Config;

use super::partitions::{Partition, Partitions};

/// This signifies the action we're currently evaluating
/// It is used for sending requests and receiving responses
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Action {
    Recalculate,
    Select,
    Wait,
}

pub struct Request {
    pub filters: Vec<Filter>,
    pub fields: Vec<GroupByField>,
}

pub type InputSender = Sender<(Request, Action)>;
pub type OutputReciever = Receiver<Result<(Partitions, Action)>>;
pub type Handle = JoinHandle<Result<(), Report>>;

pub struct Link {
    pub input_sender: InputSender,
    pub output_receiver: OutputReciever,
    pub handle: Handle,
}

pub fn run(config: &Config) -> Result<Link> {
    let database = Database::new(&config.database_path)?;
    let (input_sender, input_receiver) = unbounded();
    let (output_sender, output_receiver) = unbounded();
    let handle = std::thread::spawn(move || inner_loop(database, input_receiver, output_sender));
    Ok(Link {
        input_sender,
        output_receiver,
        handle,
    })
}

fn inner_loop(
    database: Database,
    input_receiver: Receiver<(Request, Action)>,
    output_sender: Sender<Result<(Partitions, Action)>>,
) -> Result<()> {
    loop {
        let (request, action) = input_receiver.recv()?;
        let filters = request.filters;
        let current_field = request
            .fields
            .last()
            .ok_or(eyre::eyre!("No Group By Available"))?;
        let group_by = vec![current_field.clone()];
        let query = Query {
            filters: &filters,
            group_by: &group_by,
        };
        let result = database.query(query)?;
        let partitions = calculate_partitions(&result)?;
        output_sender.send(Ok((Partitions::new(partitions), action)))?
    }
}

fn calculate_partitions<'a>(result: &[QueryResult]) -> Result<Vec<Partition>> {
    let mut partitions = Vec::new();
    for r in result.iter() {
        let partition = r.try_into()?;
        partitions.push(partition);
    }

    Ok(partitions)
}
