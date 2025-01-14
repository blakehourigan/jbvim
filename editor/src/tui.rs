use crate::{EditorMode, EditorState};
use std::error::Error;
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

pub fn update_tui(editor_state: &mut EditorState) -> Result<(), Box<dyn Error>> {
    let window_inf = InformationBar::new(&terminol::get_terminal_size());
    let mode = editor_state.editor_mode.value();

    if editor_state.editor_mode == EditorMode::Command
        && editor_state.previous_mode == EditorMode::Command
    {
        draw_command_field(&window_inf, &mode)?;
    } else {
        draw_info_tui(&window_inf)?;
        draw_mode(&window_inf, &mode)?;
        update_cursor(editor_state)?;
    }
    Ok(())
}

fn update_cursor(editor_state: &mut EditorState) -> Result<(), Box<dyn Error>> {
    match editor_state.editor_mode {
        EditorMode::Insert | EditorMode::Command => cursor::enable_bar_cursor()?,
        _ => cursor::enable_standard_cursor()?,
    }
    Ok(())
}

/// draws the tui information bar including green background and default cursor position
/// (1,1)
fn draw_info_tui(window_inf: &InformationBar) -> Result<(), Box<dyn Error>> {
    let cursor = Cursor::get_cursor_coords().expect("expecting cursor obj");
    cursor::save_cursor_position()?;
    cursor::move_cursor_to(window_inf.row, 1)?;

    // editor_data.cursor.mode(cursor::modes::bold);
    let color = Colors::Red as i32;
    cursor::set_background(color)?;

    let bar = std::iter::repeat(" ")
        .take(window_inf.length as usize)
        .collect::<String>();

    write!(io::stdout(), "{}", bar)?;

    draw_cursor_location(&window_inf, color, cursor.line, cursor.col)?;

    cursor::restore_cursor_position()?;
    cursor::reset_modes()?;
    Ok(())
}

fn draw_cursor_location(
    window_inf: &InformationBar,
    color: i32,
    line: u32,
    col: u32,
) -> Result<(), Box<dyn Error>> {
    cursor::move_cursor_to(window_inf.row, window_inf.cursor_location_col)?;

    cursor::set_background(color)?;

    write!(io::stdout(), "({},{})", line, col)?;
    Ok(())
}

fn draw_command_field(window_inf: &InformationBar, mode: &str) -> Result<(), Box<dyn Error>> {
    cursor::draw_line(
        window_inf.command_row,
        window_inf.length as usize,
        Colors::Black as i32,
    )?;
    //cursor.draw_line(row, length, color)
    cursor::move_cursor_to(window_inf.command_row, 1)?;

    write!(io::stdout(), ":")?;

    Ok(())
}

fn draw_mode(window_inf: &InformationBar, mode: &str) -> Result<(), Box<dyn Error>> {
    cursor::save_cursor_position()?;
    cursor::move_cursor_to(window_inf.command_row, window_inf.editor_mode_col)?;

    write!(io::stdout(), "{}", mode)?;
    cursor::restore_cursor_position()?;

    Ok(())
}
