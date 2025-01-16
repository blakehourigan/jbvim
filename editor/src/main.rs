use editor;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    editor::run(env::args())?;
    Ok(())
}
