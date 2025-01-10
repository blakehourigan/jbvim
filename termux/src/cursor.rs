pub mod color;
pub use color::Colors;
use std::io::{self, Write};

pub struct Cursor {
    cur_row: u32,
    cur_col: u32,
    prev_row: u32,
    prev_col: u32,
}

impl Cursor {
    // constructs new instance of cursor with current and previous row, col pairs
    // prev_row & prev_col are used to return to prev user position after a ui operation
    // home position in ascii is considered (1,1)
    pub fn new() -> Self {
        Cursor {
            cur_row: 1,
            cur_col: 1,
            prev_row: 1,
            prev_col: 1,
        }
    }

    pub fn prev_col(&self) -> u32 {
        self.prev_col
    }
    pub fn prev_row(&self) -> u32 {
        self.prev_row
    }
    pub fn move_home(&mut self) -> std::io::Result<()> {
        self.reset_coords();
        write!(io::stdout(), "\x1b[H")
    }
    pub fn move_right(&mut self, num: u32) -> std::io::Result<()> {
        self.inc_col(num);
        write!(io::stdout(), "\x1b[{num}C",)
    }
    pub fn move_left(&mut self, num: u32) -> std::io::Result<()> {
        self.dec_col(num);
        write!(io::stdout(), "\x1b[{num}D",)
    }

    pub fn move_up(&mut self, num: u32) -> std::io::Result<()> {
        self.dec_row(num);
        write!(io::stdout(), "\x1b[{num}A",)
    }

    pub fn move_down(&mut self, num: u32) -> std::io::Result<()> {
        self.inc_row(num);
        write!(io::stdout(), "\x1b[{num}B",)
    }

    pub fn move_cursor_to(&mut self, line: u32, column: u32) -> std::io::Result<()> {
        self.prev_row = self.cur_row;
        self.prev_col = self.cur_col;

        self.cur_row = line;
        self.cur_col = column;

        // syntax for the escape is line;column
        write!(io::stdout(), "\x1b[{line};{column}f")
    }

    pub fn restore_cursor_position(&mut self) -> Result<(), std::io::Error> {
        let restore_line = self.prev_row;
        let restore_column = self.prev_col;

        self.prev_col = self.cur_col;
        self.prev_row = self.cur_row;

        //restore previous coords to the current coords
        self.cur_row = restore_line;
        self.cur_col = restore_column;
        write!(io::stdout(), "\x1b[{restore_line};{restore_column}f")?;
        Ok(())
    }
    pub fn backspace(&mut self) -> std::io::Result<()> {
        self.move_left(1)?;
        write!(io::stdout(), " ")?;
        io::stdout().flush().unwrap();

        Ok(())
    }
    fn reset_coords(&mut self) {
        self.cur_row = 1;
        self.cur_col = 1;
    }
    pub fn reset_modes(&self) -> std::io::Result<()> {
        write!(io::stdout(), "\x1b[0m")
    }
    fn inc_row(&mut self, num: u32) {
        self.cur_row += num;
    }
    fn dec_row(&mut self, num: u32) {
        if self.cur_row != 1 {
            self.cur_row -= num;
        }
    }
    fn inc_col(&mut self, num: u32) {
        self.cur_col += num;
    }
    fn dec_col(&mut self, num: u32) {
        if self.cur_col != 1 {
            self.cur_col -= num;
        }
    }
    pub fn write_char(&self, character: &u8) -> std::io::Result<()> {
        write!(io::stdout(), "{}", *character as char)
    }
    pub fn set_foreground(&self, color: i32) -> std::io::Result<()> {
        write!(io::stdout(), "\x1b[38;5;{color}m")
    }
    pub fn set_background(&self, color: i32) -> std::io::Result<()> {
        write!(io::stdout(), "\x1b[48;5;{color}m")
    }
    pub fn delete_end_of_line(&self) -> std::io::Result<()> {
        write!(io::stdout(), "\x1b[0K")
    }
}
