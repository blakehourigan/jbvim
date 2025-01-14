use editor;
use editor::FileData;
use std::env;
use std::error::Error;
use std::process;

fn main() -> Result<(), Box<dyn Error>> {
    let mut file_data = FileData::build(env::args()).unwrap_or_else(|err| {
        println!("problem parsing args: {err}");
        process::exit(1);
    });
    editor::run(&mut file_data)?;
    Ok(())
}
