use std::io;

use promkit::{
    build::Builder,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    grapheme::Graphemes,
    keybind::KeyBind,
    readline::{self, State},
    EventHandleFn, Result,
};

fn main() -> Result<()> {
    let mut b = KeyBind::default();
    b.assign(vec![(
        Event::Key(KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::CONTROL,
        }),
        Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
            state.0.editor.replace(&Graphemes::from("REPLCED!!"));
            Ok(None)
        }) as Box<EventHandleFn<State>>,
    )]);
    let mut p = readline::Builder::default().handler(b).build()?;
    loop {
        let (line, exit_code) = p.run()?;
        match exit_code {
            0 => println!("result: {:?}", line),
            1 => return Ok(()),
            _ => (),
        }
    }
}
