pub mod color;
pub use color::Colors;
use std::io::{self, Write};

pub struct Cursor {
    user_row: u32,
    user_col: u32,
}

impl Cursor {
    // constructs new instance of cursor with current and previous row, col pairs
    // prev_row & prev_col are used to return to prev user position after a ui operation
    // home position in ascii is considered (1,1)
    pub fn new() -> Self {
        Cursor {
            user_row: 1,
            user_col: 1,
        }
    }

    pub fn user_col(&self) -> u32 {
        self.user_col
    }
    pub fn user_row(&self) -> u32 {
        self.user_row
    }
    fn set_user_row(&mut self, row: u32) {
        self.user_row = row;
    }
    fn set_user_col(&mut self, col: u32) {
        self.user_col = col;
    }

    pub fn move_home(&mut self) -> std::io::Result<()> {
        self.reset_coords();
        write!(io::stdout(), "\x1b[H")
    }
    pub fn move_right(&mut self, num: u32) -> std::io::Result<()> {
        self.inc_col(num);
        write!(io::stdout(), "\x1b[{num}C",)
    }
    pub fn move_left(&mut self, num: u32, command_mode: bool) -> std::io::Result<()> {
        if !command_mode {
            self.dec_col(num);
        }
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
        // syntax for the escape is line;column
        write!(io::stdout(), "\x1b[{line};{column}f")
    }

    pub fn return_newline(&mut self) -> std::io::Result<()> {
        self.set_user_col(1);

        let row = self.inc_row(1);
        let col = self.user_col();

        self.move_cursor_to(row, col)
    }
    pub fn restore_cursor_position(&mut self) -> Result<(), std::io::Error> {
        write!(io::stdout(), "\x1b[{};{}f", self.user_row, self.user_col)
    }
    pub fn backspace(&mut self, command_mode: bool) -> std::io::Result<()> {
        self.move_left(1, command_mode)?;
        write!(io::stdout(), " ")?;
        if command_mode {
            self.move_left(1, command_mode)?;
        }

        Ok(())
    }
    fn reset_coords(&mut self) {
        self.user_row = 1;
        self.user_col = 1;
    }
    pub fn reset_modes(&self) -> std::io::Result<()> {
        write!(io::stdout(), "\x1b[0m")
    }
    fn inc_row(&mut self, num: u32) -> u32 {
        self.user_row += num;
        self.user_row()
    }
    fn dec_row(&mut self, num: u32) {
        if self.user_row != 1 {
            self.user_row -= num;
        }
    }
    fn inc_col(&mut self, num: u32) {
        self.user_col += num;
    }
    fn dec_col(&mut self, num: u32) {
        if self.user_col != 1 {
            self.user_col -= num;
        }
    }
    pub fn write_char(&mut self, character: &u8, command_mode: bool) -> std::io::Result<()> {
        if !command_mode {
            self.inc_col(1);
        }
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
    pub fn draw_line(&mut self, line_num: u32, length: usize, color: i32) -> std::io::Result<()> {
        self.move_cursor_to(line_num, 1)?;

        self.set_background(color)?;

        let bar = std::iter::repeat(" ").take(length).collect::<String>();

        write!(io::stdout(), "{}", bar)?;

        self.restore_cursor_position()?;
        Ok(())
    }
}
