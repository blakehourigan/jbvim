use crate::{EditorMode, EditorState};
use std::io::{self, Write};
use terminol::cursor;
use terminol::{Colors, Cursor};

const INFO_BAR_ROW_OFFSET: u32 = 1;
const CURSOR_LOCATION_COL_OFFSET: u32 = 15;
const EDITOR_MODE_COL_OFFSET: u32 = 10;

struct InformationBar {
    length: u32,
    row: u32,
    command_row: u32,
    cursor_location_col: u32,
    editor_mode_col: u32,
}

impl InformationBar {
    fn new(term_attr: &libc::winsize) -> Self {
        InformationBar {
            length: term_attr.ws_col as u32,
            row: (term_attr.ws_row as u32) - INFO_BAR_ROW_OFFSET,
            command_row: (term_attr.ws_row as u32),
            cursor_location_col: (term_attr.ws_col as u32) - CURSOR_LOCATION_COL_OFFSET,
            editor_mode_col: EDITOR_MODE_COL_OFFSET,
        }
    }
}

pub fn update_tui(editor_state: &mut EditorState) {
    let window_inf = InformationBar::new(&terminol::get_terminal_size());
    let mode = editor_state.editor_mode.value();

    match editor_state.editor_mode {
        EditorMode::Command => {
            if editor_state.previous_mode == EditorMode::Command {
            } else {
                draw_command_field(&window_inf);
            }
        }
        _ => {
            if editor_state.previous_mode == EditorMode::Command {
                cursor::restore_cursor_position();
                editor_state.previous_mode = editor_state.editor_mode;
            } else {
                draw_info_tui(&window_inf);
                draw_mode(&window_inf, &mode);
                update_cursor(editor_state);
            }
        }
    }
}

fn update_cursor(editor_state: &mut EditorState) {
    match editor_state.editor_mode {
        EditorMode::Insert | EditorMode::Command => cursor::enable_bar_cursor(),
        _ => cursor::enable_standard_cursor(),
    }
}

/// draws the tui information bar including green background and default cursor position
/// (1,1)
fn draw_info_tui(window_inf: &InformationBar) {
    let cursor = Cursor::get_cursor_coords();
    cursor::save_cursor_position();
    cursor::move_cursor_to(window_inf.row.try_into().unwrap(), 1);

    // editor_data.cursor.mode(cursor::modes::bold);
    let color = Colors::Red as i32;
    cursor::set_background(color);

    let bar = std::iter::repeat(" ")
        .take(window_inf.length as usize)
        .collect::<String>();

    write!(io::stdout(), "{}", bar).unwrap_or_else(|e| panic!("failed io operation: {e}"));

    draw_cursor_location(&window_inf, color, cursor.line, cursor.col);

    cursor::restore_cursor_position();
    cursor::reset_modes();
}

fn draw_cursor_location(window_inf: &InformationBar, color: i32, line: usize, col: usize) {
    cursor::move_cursor_to(
        window_inf.row.try_into().unwrap(),
        window_inf.cursor_location_col.try_into().unwrap(),
    );

    cursor::set_background(color);

    write!(io::stdout(), "({},{})", line, col)
        .unwrap_or_else(|e| panic!("failed io operation: {e}"));
}

fn draw_command_field(window_inf: &InformationBar) {
    draw_line(
        window_inf.command_row.try_into().unwrap(),
        window_inf.length.try_into().unwrap(),
        Colors::Black as i32,
    );
    cursor::move_cursor_to(window_inf.command_row.try_into().unwrap(), 1);

    write!(io::stdout(), ":").unwrap_or_else(|e| panic!("failed io operation: {e}"));
}

fn draw_mode(window_inf: &InformationBar, mode: &str) {
    cursor::save_cursor_position();
    cursor::move_cursor_to(
        window_inf.command_row.try_into().unwrap(),
        window_inf.editor_mode_col.try_into().unwrap(),
    );

    write!(io::stdout(), "{}", mode).unwrap_or_else(|e| panic!("failed io operation: {e}"));
    cursor::restore_cursor_position();
}

fn draw_line(line_num: usize, length: usize, color: i32) {
    cursor::save_cursor_position();
    cursor::move_cursor_to(line_num, 1);

    cursor::set_background(color);

    let bar = std::iter::repeat(" ").take(length).collect::<String>();

    write!(io::stdout(), "{}", bar).unwrap_or_else(|e| panic!("io error{e}"));

    cursor::restore_cursor_position();
}
pub fn update_line(mut line: String, is_backspace: bool) {
    if is_backspace {
        line.push_str("  ");
    }
    cursor::save_cursor_position();
    write!(io::stdout(), "{}", line).unwrap_or_else(|e| panic!("failed io operation: {e}"));
    cursor::restore_cursor_position();
}

pub fn write_existing_file(file_contents: &String) {
    for c in file_contents.chars() {
        if c == '\n' {
            write!(io::stdout(), "\r\n").unwrap_or_else(|e| panic!("failed io operation: {e}"));
        } else {
            write!(io::stdout(), "{}", c).unwrap_or_else(|e| panic!("failed io operation: {e}"));
        }
    }
}
