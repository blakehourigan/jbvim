pub mod cursor;
pub use cursor::{Colors, Cursor};
use libc;
use std::error::Error;
use std::io::{self, Write};
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

pub struct TerminalTools {
    pub cursor: Cursor,
    pub terminal_attributes: libc::winsize,
}

impl TerminalTools {
    pub fn new() -> Self {
        TerminalTools {
            cursor: Cursor::new(),
            terminal_attributes: get_terminal_size(),
        }
    }
}

fn get_terminal_size() -> libc::winsize {
    let mut terminal_window_attr = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe {
        libc::ioctl(
            libc::STDIN_FILENO,
            libc::TIOCGWINSZ,
            &mut terminal_window_attr,
        );
    }
    terminal_window_attr
}
/// ulilizes termios from libc to enable raw mode in the terminal. this function disables the
/// icanon and echo flags in the c_lflag register to disable canonical mode and echo terminal
/// functionality
pub fn enable_raw_mode() -> Result<(), Box<dyn Error>> {
    let mut termios = Termios::from_fd(libc::STDIN_FILENO).unwrap();
    termios::cfmakeraw(&mut termios);

    // disable canonical terminal mode and typing echo to only print chars we want.
    // e.g. ascii in insert mode
    termios.c_lflag &= !(ICANON | ECHO);

    match tcsetattr(libc::STDIN_FILENO, TCSANOW, &termios) {
        Ok(_) => (),
        Err(e) => write!(io::stdout(), "error, {}", e)?,
    };
    Ok(())
}

/// enables the alternate buffer and enters it to create a clean new buffer for the program.
/// This saves the terminal buffer that the program was launched with and allows for return
/// to this buffer later.
pub fn enable_alternate_buffer() -> Result<(), std::io::Error> {
    write!(io::stdout(), "\x1b[?1049h")?;
    Ok(())
}

/// disables the alternate buffer and returns to the buffer used to launch the
/// program.
pub fn disable_alternate_buffer() -> Result<(), std::io::Error> {
    write!(io::stdout(), "\x1b[?1049l")?;
    Ok(())
}
pub fn clear_screen() -> std::io::Result<()> {
    write!(io::stdout(), "\x1b[2J")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
