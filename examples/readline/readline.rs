use anyhow::Result;

use promkit::{
    editor::{TextBuilder, TextEditorBuilder},
    Prompt,
};

fn main() -> Result<()> {
    let mut p = Prompt::new(vec![
        Box::new(TextBuilder::new("hello world").build()?),
        Box::new(TextEditorBuilder::new().build()?),
        Box::new(TextEditorBuilder::new().build()?),
    ]);
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
