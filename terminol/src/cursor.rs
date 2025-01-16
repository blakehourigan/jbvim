pub mod color;
pub use color::Colors;
use std::io::{self, BufRead, Write};

#[derive(Debug)]
pub struct Cursor {
    pub line: u32,
    pub col: u32,
}

impl Cursor {
    fn new(line: u32, col: u32) -> Cursor {
        Cursor { line, col }
    }
    /// constructs new instance of cursor with current and curosor row and col
    /// using ascii escape character "\x1b[6n". stdout is flushed and stdin response
    /// from terminal is read into cursor instance.
    pub fn get_cursor_coords() -> Cursor {
        write!(io::stdout(), "\x1b[6n")
            .unwrap_or_else(|e| panic!("io error occurred during write: {e}"));

        io::stdout()
            .flush()
            .unwrap_or_else(|e| panic!("io error occurred during flush: {e}"));

        let mut v: Vec<u8> = Vec::with_capacity(30);
        let mut reader = io::stdin().lock();
        let _ = reader.read_until(b'R', &mut v);

        let mut iterator = v.into_iter();
        // take iterator by reference so that we can use the
        // remaining elements in the iterator once we find the line
        let line: u32 = iterator
            .by_ref()
            .map(|c| c as char)
            .take_while(|c| *c != ';')
            .filter(|i| i.is_numeric())
            .map(|c| c.to_digit(10).expect("not a number!"))
            .fold(0, |acc, elem| acc * 10 + elem);

        let col: u32 = iterator
            .by_ref()
            .map(|c| c as char)
            .filter(|i| i.is_numeric())
            .map(|c| c.to_digit(10).expect("not a number!"))
            .fold(0, |acc, elem| acc * 10 + elem);

        Cursor::new(line, col)
    }
}
pub fn enable_bar_cursor() {
    write!(io::stdout(), "\x1b[6 q",).unwrap_or_else(|e| panic!("io error{e}"))
}
pub fn enable_standard_cursor() {
    write!(io::stdout(), "\x1b[0 q",).unwrap_or_else(|e| panic!("io error{e}"))
}
pub fn move_right(num: u32) {
    write!(io::stdout(), "\x1b[{num}C",).unwrap_or_else(|e| panic!("io error{e}"))
}
pub fn move_left(num: u32) {
    write!(io::stdout(), "\x1b[{num}D",).unwrap_or_else(|e| panic!("io error{e}"))
}
pub fn move_up(num: u32) {
    write!(io::stdout(), "\x1b[{num}A",).unwrap_or_else(|e| panic!("io error{e}"))
}

pub fn move_down(num: u32) {
    write!(io::stdout(), "\x1b[{num}B",).unwrap_or_else(|e| panic!("io error{e}"))
}

pub fn move_cursor_to(line: u32, column: u32) {
    // syntax for the escape is line;column
    write!(io::stdout(), "\x1b[{line};{column}f").unwrap_or_else(|e| panic!("io error{e}"))
}
pub fn move_home() {
    write!(io::stdout(), "\x1b[H").unwrap_or_else(|e| panic!("io error{e}"))
}
pub fn return_newline() {
    let cursor = Cursor::get_cursor_coords();
    move_cursor_to(&cursor.line + 1, cursor.col)
}
pub fn save_cursor_position() {
    write!(io::stdout(), "\x1b[s").unwrap_or_else(|e| panic!("io error{e}"))
}
pub fn restore_cursor_position() {
    write!(io::stdout(), "\x1b[u").unwrap_or_else(|e| panic!("io error{e}"))
}
pub fn backspace() {
    move_left(1);
    write!(io::stdout(), " ").unwrap_or_else(|e| panic!("io error{e}"));
    move_left(1);
}
pub fn write_char(character: &u8) {
    write!(io::stdout(), "{}", *character as char).unwrap_or_else(|e| panic!("io error{e}"));
}
pub fn set_foreground(color: i32) {
    write!(io::stdout(), "\x1b[38;5;{color}m").unwrap_or_else(|e| panic!("io error{e}"));
}
pub fn set_background(color: i32) {
    write!(io::stdout(), "\x1b[48;5;{color}m").unwrap_or_else(|e| panic!("io error{e}"));
}
pub fn delete_end_of_line() {
    write!(io::stdout(), "\x1b[0K").unwrap_or_else(|e| panic!("io error{e}"));
}
pub fn reset_modes() {
    write!(io::stdout(), "\x1b[0m").unwrap_or_else(|e| panic!("io error{e}"));
}
pub fn make_invisible() {
    write!(io::stdout(), "\x1b[?25h").unwrap_or_else(|e| panic!("io error{e}"));
}
pub fn make_visible() {
    write!(io::stdout(), "\x1b[28m").unwrap_or_else(|e| panic!("io error{e}"));
}

pub fn draw_line(line_num: u32, length: usize, color: i32) {
    save_cursor_position();
    move_cursor_to(line_num, 1);

    set_background(color);

    let bar = std::iter::repeat(" ").take(length).collect::<String>();

    write!(io::stdout(), "{}", bar).unwrap_or_else(|e| panic!("io error{e}"));

    restore_cursor_position();
}
