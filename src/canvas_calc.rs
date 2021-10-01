//! Runs a continuous thread to calculate the canvas.
//! Receives as input the current gui app state and size via a channel,
//! Then performs the SQLite query
//! Then performs the calculation to the `TreeMap`
//! And finally uses a channel to submit the result back to the UI
//! Runs its own connection to the SQLite database.

use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::thread::JoinHandle;

use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui::Rect as EguiRect;
use eyre::{bail, Report, Result};
use treemap::{Mappable, Rect, TreemapLayout};

use crate::database::{
    query::{DynamicType, Filter, GroupByField, Query, ValueField},
    query_result::QueryResult,
    Database,
};
use crate::gui::state::State;
use crate::types::Config;

#[derive(Debug, Hash, Clone)]
pub enum Value {
    Number(usize),
    String(String),
    Bool(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => f.write_str(&n.to_string()),
            Value::Bool(n) => f.write_str(&n.to_string()),
            Value::String(n) => f.write_str(&n),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub field: GroupByField,
    pub count: usize,
    pub value: Value,
    /// A TreeMap Rect
    pub rect: Rect,
}

impl Partition {
    /// Perform rect conversion from TreeMap to Egui
    pub fn layout_rect(&self) -> EguiRect {
        use eframe::egui::pos2;
        EguiRect {
            min: pos2(self.rect.x as f32, self.rect.y as f32),
            max: pos2(
                self.rect.x as f32 + self.rect.w as f32,
                self.rect.y as f32 + self.rect.h as f32,
            ),
        }
    }
}

/// A small NewType so that we can keep all the `TreeMap` code in here and don't
/// have to do the layout calculation in a widget.
pub struct Partitions {
    pub items: Vec<Partition>,
    pub selected: Option<Partition>,
}

impl Partitions {
    pub fn new(items: Vec<Partition>) -> Self {
        Self {
            items,
            selected: None,
        }
    }

    /// Update the layout information in the partitions
    /// based on the current size
    pub fn update_layout(&mut self, rect: EguiRect) {
        let layout = TreemapLayout::new();
        let bounds = Rect::from_points(
            rect.left() as f64,
            rect.top() as f64,
            rect.width() as f64,
            rect.height() as f64,
        );
        layout.layout_items(&mut self.items, bounds);
    }
}

impl Mappable for Partition {
    fn size(&self) -> f64 {
        self.count as f64
    }

    fn bounds(&self) -> &Rect {
        &self.rect
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.rect = bounds;
    }
}

impl<'a> TryFrom<&'a QueryResult<'a>> for Partition {
    type Error = Report;
    fn try_from(r: &'a QueryResult<'a>) -> Result<Self> {
        // so far we can only support one group by at a time.
        // at least in here. The queries support it
        let field = r
            .values
            .first()
            .ok_or(eyre::eyre!("No group by fields available"))?;

        let value = match (field.is_bool(), field.is_str(), field.is_usize()) {
            (true, false, false) => Value::Bool(*field.as_bool()),
            (false, true, false) => Value::String(field.as_str().to_string()),
            (false, false, true) => Value::Number(*field.as_usize()),
            _ => bail!("Invalid field: {:?}", &field),
        };

        Ok(Partition {
            field: field.as_field(),
            count: r.count,
            value,
            rect: Rect::new(),
        })
    }
}

pub type InputSender = Sender<State>;
pub type OutputReciever = Receiver<Result<Partitions>>;
pub type Handle = JoinHandle<Result<(), Report>>;

pub fn run(config: &Config) -> Result<(InputSender, OutputReciever, Handle)> {
    let database = Database::new(&config.database_path)?;
    let (input_sender, input_receiver) = unbounded();
    let (output_sender, output_receiver) = unbounded();
    let handle = std::thread::spawn(move || inner_loop(database, input_receiver, output_sender));
    Ok((input_sender, output_receiver, handle))
}

fn inner_loop(
    database: Database,
    input_receiver: Receiver<State>,
    output_sender: Sender<Result<Partitions>>,
) -> Result<()> {
    loop {
        let task = input_receiver.recv()?;
        let filters = convert_filters(&task);
        let group_by = vec![GroupByField::Year];
        let query = Query {
            filters: &filters,
            group_by: &group_by,
        };
        let result = database.query(query)?;
        let partitions = calculate_partitions(&result)?;
        output_sender.send(Ok(Partitions::new(partitions)))?
    }
}

fn calculate_partitions<'a>(result: &[QueryResult<'a>]) -> Result<Vec<Partition>> {
    let mut partitions = Vec::new();
    for r in result.iter() {
        let partition = r.try_into()?;
        partitions.push(partition);
    }

    Ok(partitions)
}

fn convert_filters<'a>(state: &'a State) -> Vec<Filter<'a>> {
    let mut filters = Vec::new();
    if let Some(ref n) = state.domain_filter {
        filters.push(Filter::Like(ValueField::SenderDomain(n.into())));
    }
    if let Some(n) = state.year_filter {
        filters.push(Filter::Is(ValueField::Year(n)));
    }
    filters
}