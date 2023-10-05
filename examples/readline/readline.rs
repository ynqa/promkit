use anyhow::Result;

use promkit::{editor::TextEditorBuilder, Prompt};

fn main() -> Result<()> {
    let mut p = Prompt::new(vec![
        Box::new(TextEditorBuilder::new().build()?),
        Box::new(TextEditorBuilder::new().build()?),
        Box::new(TextEditorBuilder::new().build()?),
    ]);
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
