use std::{
    io::Write,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use crate::terminal::TerminalSize;
use portable_pty::{Child, MasterPty};

pub struct Session {
    pub child: Box<dyn Child + Send + Sync>,
    pub master: Box<dyn MasterPty + Send>,
    pub writer: Box<dyn Write + Send>,
    pub output: Arc<Mutex<Vec<u8>>>,
    pub reader_thread: Option<JoinHandle<()>>,
    pub size: TerminalSize,
}
