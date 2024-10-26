use promkit::preset::readline::Readline;
use std::io;

fn main() -> anyhow::Result<()> {
    let mut p = Readline::default().prompt(io::stdout())?;

    loop {
        match p.run() {
            Ok(cmd) => {
                println!("result: {:?}", cmd);
            }
            Err(_) => {
                println!("Bye!");
                break;
            }
        }
    }
    Ok(())
}
