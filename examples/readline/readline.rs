use anyhow::Result;
use promkit::Prompt;

fn main() -> Result<()> {
    let mut p = Prompt::new();
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
