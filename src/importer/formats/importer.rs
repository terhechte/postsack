use super::ImporterFormat;

use crossbeam_channel::{self, Sender};
use std::thread::JoinHandle;

pub enum Message {}

pub struct Importer<FORMAT: ImporterFormat> {
    format: FORMAT,
    sender: Sender<Message>,
}
