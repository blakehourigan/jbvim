use editor;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    editor::run()?;
    Ok(())
}
