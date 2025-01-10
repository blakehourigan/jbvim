use std::error::Error;
use std::io::{self, Read, Write};
use termux::{cursor, Colors};

struct InformationBar {
    length: u32,
    row: u32,
    command_row: u32,
    cursor_location_col: u32,
    editor_mode_col: u32,
    message_col: u32,
}

impl InformationBar {
    fn new(term_attr: &libc::winsize) -> Self {
        InformationBar {
            length: term_attr.ws_col as u32,
            row: (term_attr.ws_row as u32) - 1,
            command_row: (term_attr.ws_row as u32),
            cursor_location_col: (term_attr.ws_col as u32) - 15,
            editor_mode_col: 10,
            message_col: 30,
        }
    }
}
/// draws the tui information bar including green background and default cursor position
/// (1,1)
pub fn draw_info_tui(editor_data: &mut crate::EditorData) -> Result<(), Box<dyn Error>> {
    let window_inf = InformationBar::new(&editor_data.terminal_attributes);

    editor_data.cursor.move_cursor_to(window_inf.row, 1)?;
    // editor_data.cursor.mode(cursor::modes::bold);
    let color = Colors::Plum4 as i32;
    editor_data.cursor.set_background(color)?;

    let bar = std::iter::repeat(" ")
        .take(window_inf.length as usize)
        .collect::<String>();

    write!(io::stdout(), "{}", bar)?;

    //restore to home position before next draw op
    editor_data.cursor.restore_cursor_position()?;

    draw_cursor_location(editor_data, &window_inf, color)?;

    editor_data.cursor.restore_cursor_position()?;

    editor_data.cursor.reset_modes()?;
    Ok(())
}

fn draw_cursor_location(
    editor_data: &mut crate::EditorData,
    window_inf: &InformationBar,
    color: i32,
) -> Result<(), Box<dyn Error>> {
    editor_data
        .cursor
        .move_cursor_to(window_inf.row, window_inf.cursor_location_col)?;

    editor_data.cursor.set_background(color)?;

    write!(
        io::stdout(),
        "({},{})",
        editor_data.cursor.prev_row(),
        editor_data.cursor.prev_col()
    )?;
    Ok(())
}

fn draw_line(cursor: &mut cursor::Cursor, color: i32, line_num: u32, length: usize) {
    cursor.move_cursor_to(line_num, 1);

    cursor.set_background(color);

    let bar = std::iter::repeat(" ").take(length).collect::<String>();

    write!(io::stdout(), "{}", bar);

    cursor.restore_cursor_position();
}

pub fn enable_command_field(editor_data: &mut crate::EditorData) -> Result<(), Box<dyn Error>> {
    let window_inf = InformationBar::new(&editor_data.terminal_attributes);
    //cursor.draw_line(row, length, color)
    draw_line(
        &mut editor_data.cursor,
        Colors::Grey as i32,
        window_inf.command_row,
        window_inf.length as usize,
    );

    editor_data
        .cursor
        .move_cursor_to(window_inf.command_row, 1)?;

    write!(io::stdout(), "{}", editor_data.character_buffer[0] as char)?;
    io::stdout().flush().unwrap();

    let mut command = String::new();

    loop {
        editor_data
            .reader
            .read_exact(&mut editor_data.character_buffer)?;
        match crate::command_mode_handler(editor_data, &mut command) {
            Ok(Option::None) => break,
            _ => (),
        }

        io::stdout().flush().unwrap();
    }
    editor_data.cursor.restore_cursor_position()?;
    io::stdout().flush().unwrap();

    Ok(())
}

//fn draw_message(
//    &self,
//    editor_data: &mut crate::EditorData,
//    msg: String,
//) -> Result<(), Box<dyn Error>> {
//    editor_data
//        .cursor
//        .move_cursor_to(self.row, self.message_col)?;
//    write!(io::stdout(), "\x1b[42m")?;
//    write!(io::stdout(), "{}", msg)?;
//    editor_data.cursor.reset_modes()?;
//    editor_data.cursor.restore_cursor_position()?;
//    Ok(())
//}
