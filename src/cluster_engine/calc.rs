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

pub enum Request {
    Grouped {
        filters: Vec<Filter>,
        group_by: GroupByField,
    },
    Normal {
        filters: Vec<Filter>,
        fields: Vec<GroupByField>,
    },
}

pub type InputSender<Context> = Sender<(Request, Context)>;
pub type OutputReciever<Context> = Receiver<Result<(Partitions, Context)>>;
pub type Handle = JoinHandle<Result<(), Report>>;

pub struct Link<Context: Send + 'static> {
    pub input_sender: InputSender<Context>,
    pub output_receiver: OutputReciever<Context>,
    pub handle: Handle,
}

pub fn run<Context: Send + Sync + 'static>(config: &Config) -> Result<Link<Context>> {
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

fn inner_loop<Context: Send + Sync + 'static>(
    database: Database,
    input_receiver: Receiver<(Request, Context)>,
    output_sender: Sender<Result<(Partitions, Context)>>,
) -> Result<()> {
    loop {
        let (request, context) = input_receiver.recv()?;
        let query: Query = (&request).into();
        let result = database.query(query)?;
        let partitions = calculate_partitions(&result)?;
        output_sender.send(Ok((Partitions::new(partitions), context)))?
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

impl<'a> From<&'a Request> for Query<'a> {
    fn from(request: &'a Request) -> Self {
        match request {
            Request::Grouped { filters, group_by } => Query::Grouped {
                filters: &filters,
                group_by: &group_by,
            },
            Request::Normal { fields, filters } => Query::Normal {
                fields: &fields,
                filters: &filters,
            },
        }
    }
}
