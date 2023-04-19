use std::io;

use promkit::{
    build::Builder,
    keybind::KeyBind,
    readline::{self, handler, State},
    EventHandleFn, Result,
};

fn main() -> Result<()> {
    let b = KeyBind::<State> {
        handle_input: Some(Box::new(
            |_, input: Option<char>, out: &mut io::Stdout, state: &mut State| {
                if let Some(input) = input {
                    let input = if input.is_uppercase() {
                        input.to_ascii_lowercase()
                    } else {
                        input.to_ascii_uppercase()
                    };
                    handler::input_char()(None, Some(input), out, state)
                } else {
                    Ok(None)
                }
            },
        ) as Box<EventHandleFn<State>>),
        ..Default::default()
    };
    let mut p = readline::Builder::default().handler(b).build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
