use std::{fs::File, io::Read};

use promkit::{json::JsonStream, preset::json::Json, serde_json::Deserializer};

fn main() -> anyhow::Result<()> {
  let mut ret = String::new();
    File::open("/Users/eufy/workspace/github.com/ynqa/promkit-async/examples/test.json")?.read_to_string(&mut ret)?;
    let stream = JsonStream::new(
        Deserializer::from_str(
          &ret,  
        )
        .into_iter::<serde_json::Value>()
        .filter_map(serde_json::Result::ok),
        None,
    );

    let mut p = Json::new(stream)
        .title("JSON viewer")
        .json_lines(5)
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
