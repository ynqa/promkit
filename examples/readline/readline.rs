use anyhow::Result;

use promkit::{editor::text::TextEditor, Prompt};

fn main() -> Result<()> {
    let texteditor = TextEditor::new();
    let mut p = Prompt::new(vec![Box::new(texteditor)]);
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
