use crate::EditorState;
use std::error::Error;
use std::io::{self, Write};
use termux::{Colors, TerminalTools};

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

pub fn update_tui(
    editor_state: &mut EditorState,
    terminal_tools: &mut TerminalTools,
) -> Result<(), Box<dyn Error>> {
    let window_inf = InformationBar::new(&terminal_tools.terminal_attributes);
    let mode = editor_state.editor_mode.value();

    match editor_state.editor_mode {
        crate::EditorMode::Command => match editor_state.previous_mode {
            crate::EditorMode::Command => (),
            _ => draw_command_field(terminal_tools, &window_inf, &mode)?,
        },
        _ => {
            if editor_state.previous_mode.value() == editor_state.editor_mode.value() {
                draw_info_tui(terminal_tools, &window_inf)?;
            } else {
                draw_info_tui(terminal_tools, &window_inf)?;
                draw_command_bar(terminal_tools, &window_inf, &mode)?;
            }
        }
    }
    Ok(())
}

/// draws the tui information bar including green background and default cursor position
/// (1,1)
fn draw_info_tui(
    terminal_tools: &mut TerminalTools,
    window_inf: &InformationBar,
) -> Result<(), Box<dyn Error>> {
    terminal_tools.cursor.move_cursor_to(window_inf.row, 1)?;
    // editor_data.cursor.mode(cursor::modes::bold);
    let color = Colors::Red as i32;
    terminal_tools.cursor.set_background(color)?;

    let bar = std::iter::repeat(" ")
        .take(window_inf.length as usize)
        .collect::<String>();

    write!(io::stdout(), "{}", bar)?;

    //restore to home position before next draw op
    terminal_tools.cursor.restore_cursor_position()?;

    draw_cursor_location(terminal_tools, &window_inf, color)?;

    terminal_tools.cursor.restore_cursor_position()?;

    terminal_tools.cursor.reset_modes()?;
    Ok(())
}

fn draw_cursor_location(
    terminal_tools: &mut TerminalTools,
    window_inf: &InformationBar,
    color: i32,
) -> Result<(), Box<dyn Error>> {
    terminal_tools
        .cursor
        .move_cursor_to(window_inf.row, window_inf.cursor_location_col)?;

    terminal_tools.cursor.set_background(color)?;

    write!(
        io::stdout(),
        "({},{})",
        terminal_tools.cursor.user_row(),
        terminal_tools.cursor.user_col()
    )?;
    Ok(())
}

fn draw_command_field(
    terminal_tools: &mut TerminalTools,
    window_inf: &InformationBar,
    mode: &str,
) -> Result<(), Box<dyn Error>> {
    draw_command_bar(terminal_tools, window_inf, mode)?;
    //cursor.draw_line(row, length, color)

    terminal_tools
        .cursor
        .move_cursor_to(window_inf.command_row, 1)?;

    write!(io::stdout(), ":")?;

    Ok(())
}

fn draw_command_bar(
    terminal_tools: &mut TerminalTools,
    window_inf: &InformationBar,
    mode: &str,
) -> Result<(), Box<dyn Error>> {
    //cursor.draw_line(row, length, color)
    terminal_tools.cursor.draw_line(
        window_inf.command_row,
        window_inf.length as usize,
        Colors::Black as i32,
    )?;
    draw_mode(terminal_tools, window_inf, mode)?;

    Ok(())
}

fn draw_mode(
    terminal_tools: &mut TerminalTools,
    window_inf: &InformationBar,
    mode: &str,
) -> Result<(), Box<dyn Error>> {
    terminal_tools
        .cursor
        .move_cursor_to(window_inf.command_row, window_inf.editor_mode_col)?;

    write!(io::stdout(), "{}", mode)?;
    terminal_tools.cursor.restore_cursor_position()?;

    Ok(())
}
