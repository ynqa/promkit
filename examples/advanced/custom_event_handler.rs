use std::collections::HashMap;
use std::io;

use promkit::{
    build::Builder,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    handler,
    readline::{self, State},
    ExitCode, Handler, Result,
};

#[derive(Default)]
struct MyHandler {
    event_counter: HashMap<Event, usize>,
}

impl Handler<State> for MyHandler {
    fn handle(
        &mut self,
        ev: Event,
        out: &mut io::Stdout,
        state: &mut State,
    ) -> Result<Option<ExitCode>> {
        *self.event_counter.entry(ev).or_insert(0) += 1;
        match ev {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
            }) => handler::enter()(None, None, out, state),
            _ => Ok(None),
        }
    }
}

fn main() -> Result<()> {
    let h = MyHandler::default();
    let mut p = readline::Builder::default().handler(h).label("").build()?;
    loop {
        let (_, exit_code) = p.run()?;
        if exit_code == 0 {
            break;
        }
    }
    println!(
        "result: {:?}",
        p.handler
            .borrow()
            .downcast_ref::<MyHandler>()
            .unwrap()
            .event_counter,
    );
    Ok(())
}
