pub mod cursor;
pub use cursor::{Colors, Cursor};
use libc;
use std::io::{self, Write};
use termios::{
    tcsetattr, Termios, BRKINT, CS8, CSIZE, ECHO, ECHONL, ICANON, ICRNL, IEXTEN, IGNBRK, IGNCR,
    INLCR, ISIG, ISTRIP, IXON, OPOST, PARENB, PARMRK, TCSANOW,
};

pub fn get_terminal_size() -> libc::winsize {
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
pub fn enable_raw_mode() -> Termios {
    let mut termios = Termios::from_fd(libc::STDIN_FILENO).unwrap();
    let original_termios = termios;
    //termios::cfmakeraw(&mut termios);

    // disable canonical terminal mode and typing echo to only print chars we want.
    // e.g. ascii in insert mode
    termios.c_iflag &= !(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON);
    termios.c_oflag &= !OPOST;
    termios.c_lflag &= !(ECHO | ECHONL | ICANON | ISIG | IEXTEN);
    termios.c_lflag &= !(ICANON | ECHO);
    termios.c_cflag &= !(CSIZE | PARENB);
    termios.c_cflag |= CS8;

    tcsetattr(libc::STDIN_FILENO, TCSANOW, &termios)
        .unwrap_or_else(|e| panic!("error writing to the std output: {e}"));

    original_termios
}
pub fn disable_raw_mode(original_settings: &Termios) {
    tcsetattr(libc::STDIN_FILENO, TCSANOW, &original_settings)
        .unwrap_or_else(|e| panic!("std io error, {e}"))
}

/// enables the alternate buffer and enters it to create a clean new buffer for the program.
/// This saves the terminal buffer that the program was launched with and allows for return
/// to this buffer later.
pub fn enable_alternate_buffer() {
    write!(io::stdout(), "\x1b[?1049h").unwrap_or_else(|e| panic!("std io error, {e}"))
}

/// disables the alternate buffer and returns to the buffer used to launch the
/// program.
pub fn disable_alternate_buffer() {
    write!(io::stdout(), "\x1b[?1049l").unwrap_or_else(|e| panic!("std io error, {e}"))
}
pub fn clear_screen() {
    write!(io::stdout(), "\x1b[2J").unwrap_or_else(|e| panic!("std io error, {e}"))
}
pub fn clear_end_of_line() {
    write!(io::stdout(), "\x1b[0K").unwrap_or_else(|e| panic!("std io error, {e}"))
}
pub fn clear_end_of_screen() {
    write!(io::stdout(), "\x1b[0J").unwrap_or_else(|e| panic!("std io error, {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_size() {
        print!("{}", get_terminal_size().ws_col);
    }
}
