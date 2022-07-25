use promkit::{build::Builder, tree, Result};

fn main() -> Result<()> {
    let mut p = tree::Builder::default().build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
