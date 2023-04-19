use std::io;

use promkit::{
    build::Builder,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    handler,
    readline::{self, State},
    Handler, Result,
};

#[derive(Default)]
struct MyHandler {
    event_counter: usize,
}

impl Handler<State> for MyHandler {
    fn handle(&mut self, ev: Event, out: &mut io::Stdout, state: &mut State) -> Result<bool> {
        self.event_counter += 1;
        match ev {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            }) => handler::enter()(None, None, out, state),
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => handler::interrupt()(None, None, out, state),
            _ => Ok(false),
        }
    }
}

fn main() -> Result<()> {
    let h = MyHandler::default();
    let mut p = readline::Builder::default().handler(h).build()?;
    loop {
        p.run()?;
        println!(
            "How many times did the events happen?: {:?}",
            p.handler
                .borrow()
                .downcast_ref::<MyHandler>()
                .unwrap()
                .event_counter,
        );
    }
}
