pub mod color;
pub use color::Colors;
use std::io::{self, Read, Write};

#[derive(Debug)]
pub struct Cursor {
    pub line: u32,
    pub col: u32,
}

impl Cursor {
    // constructs new instance of cursor with current and previous row, col pairs
    // prev_row & prev_col are used to return to prev user position after a ui operation
    // home position in ascii is considered (1,1)
    pub fn get_cursor_coords() -> Result<Self, std::io::Error> {
        write!(io::stdout(), "\x1b[6n")?;
        io::stdout().flush().unwrap();

        let mut v = vec![0u8; 8];
        let _ = io::stdin().read(&mut v);
        let row_col: Vec<_> = v
            .into_iter()
            .map(|c| c as char)
            .filter(|i| i.is_numeric())
            .map(|c| c.to_digit(10).expect("not a number!"))
            .collect();
        let line = row_col[0];
        let col = row_col[1];
        Ok(Cursor { line, col })
    }
}
pub fn enable_bar_cursor() -> std::io::Result<()> {
    write!(io::stdout(), "\x1b[6 q",)
}
pub fn enable_standard_cursor() -> std::io::Result<()> {
    write!(io::stdout(), "\x1b[0 q",)
}
pub fn move_right(num: u32) -> std::io::Result<()> {
    write!(io::stdout(), "\x1b[{num}C",)
}
pub fn move_left(num: u32) -> std::io::Result<()> {
    write!(io::stdout(), "\x1b[{num}D",)
}
pub fn move_up(num: u32) -> std::io::Result<()> {
    write!(io::stdout(), "\x1b[{num}A",)
}

pub fn move_down(num: u32) -> std::io::Result<()> {
    write!(io::stdout(), "\x1b[{num}B",)
}

pub fn move_cursor_to(line: u32, column: u32) -> std::io::Result<()> {
    // syntax for the escape is line;column
    write!(io::stdout(), "\x1b[{line};{column}f")
}
pub fn move_home() -> std::io::Result<()> {
    write!(io::stdout(), "\x1b[H")
}
pub fn return_newline() -> std::io::Result<()> {
    let cursor = Cursor::get_cursor_coords().unwrap();
    move_cursor_to(&cursor.line + 1, cursor.col)
}
pub fn save_cursor_position() -> Result<(), std::io::Error> {
    write!(io::stdout(), "\x1b[s")
}
pub fn restore_cursor_position() -> Result<(), std::io::Error> {
    write!(io::stdout(), "\x1b[u")
}
pub fn backspace() -> std::io::Result<()> {
    move_left(1)?;
    write!(io::stdout(), " ")?;

    Ok(())
}
pub fn write_char(character: &u8) -> std::io::Result<()> {
    write!(io::stdout(), "{}", *character as char)
}
pub fn set_foreground(color: i32) -> std::io::Result<()> {
    write!(io::stdout(), "\x1b[38;5;{color}m")
}
pub fn set_background(color: i32) -> std::io::Result<()> {
    write!(io::stdout(), "\x1b[48;5;{color}m")
}
pub fn delete_end_of_line() -> std::io::Result<()> {
    write!(io::stdout(), "\x1b[0K")
}
pub fn reset_modes() -> std::io::Result<()> {
    write!(io::stdout(), "\x1b[0m")
}

pub fn draw_line(line_num: u32, length: usize, color: i32) -> std::io::Result<()> {
    save_cursor_position()?;
    move_cursor_to(line_num, 1)?;

    set_background(color)?;

    let bar = std::iter::repeat(" ").take(length).collect::<String>();

    write!(io::stdout(), "{}", bar)?;

    restore_cursor_position()?;
    Ok(())
}
