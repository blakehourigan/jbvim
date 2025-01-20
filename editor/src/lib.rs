pub mod config;
pub use config::FileData;
mod tui;
use config::{EditorMode, EditorState};
use std::error::Error;
use std::io::{self, Read, Write};
use std::process;
use std::{env, usize};
use terminol::{cursor, Cursor};
use termios::Termios;

pub fn run(cmd_args: env::Args) -> Result<Termios, Box<dyn Error>> {
    let mut editor_state = EditorState::new(EditorMode::Normal, EditorMode::Normal);
    let original_settings = terminol::enable_raw_mode();

    cursor::move_home();
    cursor::save_cursor_position();
    terminol::enable_alternate_buffer();
    terminol::clear_screen();

    tui::update_tui(&mut editor_state);
    io::stdout().flush().unwrap();

    let mut file_data = FileData::build(cmd_args).unwrap_or_else(|err| {
        println!("problem parsing args: {err}");
        graceful_exit(&original_settings);
        process::exit(1)
    });

    cursor::restore_cursor_position();
    io::stdout()
        .flush()
        .unwrap_or_else(|e| panic!("io error occurred during flush: {e}"));

    let mut command = String::new();

    loop {
        let mut input = [0u8; 3];
        // opening reader gets rid of the shell prompt guy
        if editor_state.editor_mode != EditorMode::ShutDown {
            io::stdin().read(&mut input)?;
        }
        match editor_state.get_current_mode() {
            EditorMode::Normal => normal_mode_handler(&input, &mut editor_state, &mut file_data),
            EditorMode::Insert => insert_mode_handler(&input, &mut editor_state, &mut file_data),
            EditorMode::Visual => normal_mode_handler(&input, &mut editor_state, &mut file_data),
            EditorMode::Command => {
                command_mode_handler(&input, &mut editor_state, &mut file_data, &mut command)
            }
            EditorMode::ShutDown => {
                graceful_exit(&original_settings);
                break;
            }
        };
        tui::update_tui(&mut editor_state);

        io::stdout()
            .flush()
            .unwrap_or_else(|e| panic!("io error occurred during flush: {e}"));
    }
    Ok(original_settings)
}

fn normal_mode_handler(input: &[u8], editor_state: &mut EditorState, file_data: &mut FileData) {
    cursor::enable_standard_cursor();
    match input[0] {
        b':' => {
            editor_state.update_editor_mode(EditorMode::Command);
        }
        b'j' | b'k' | b'l' | b'h' => {
            basic_movement_handler(input, file_data, editor_state);
        }
        b'i' => {
            editor_state.update_editor_mode(EditorMode::Insert);
        }
        b'v' => editor_state.update_editor_mode(EditorMode::Visual),
        b'0' => cursor::move_cursor_to(Cursor::get_cursor_coords().line, 1),
        b'a' => {
            cursor::move_right(1);
            file_data.file_contents_buffer.move_cursor_right();
            editor_state.update_editor_mode(EditorMode::Insert);
        }
        // return/enter
        13 => cursor::move_down(1),
        //backspace
        127 => cursor::move_left(1),
        27 => basic_movement_handler(&input, file_data, editor_state),
        _ => (),
    }
}

fn insert_mode_handler(input: &[u8], editor_state: &mut EditorState, file_data: &mut FileData) {
    match input[0] {
        // return/enter
        13 => {
            file_data.file_contents_buffer.insert_left('\n');
            file_data.file_contents_buffer.insert_left('\r');
            cursor::return_newline();
        }
        //backspace
        127 => {
            file_data.file_contents_buffer.delete_char();
            cursor::backspace();
            // grab from 1 after the gaps end until the next newline or until the eof
            let line_end = file_data.file_contents_buffer.grab_to_line_end();
            // tell tui that we need to update the line we're on with the content we just got
            // after our current position
            cursor::write_char(&input[0]);
            tui::update_line(line_end, true);
        }
        // <C-c> | Esc
        3 => {
            editor_state.update_editor_mode(EditorMode::Normal);
        }
        27 => basic_movement_handler(input, file_data, editor_state),
        _ => {
            if file_data.file_contents_buffer.is_line_end() {
                file_data.file_contents_buffer.insert_left(input[0] as char);
                cursor::write_char(&input[0]);
            } else {
                // insert the char
                file_data.file_contents_buffer.insert_left(input[0] as char);
                // grab from 1 after the gaps end until the next newline or until the eof
                let line_end = file_data.file_contents_buffer.grab_to_line_end();
                // tell tui that we need to update the line we're on with the content we just got
                // after our current position
                cursor::write_char(&input[0]);
                tui::update_line(line_end, false);
            }
        }
    };
}

fn basic_movement_handler(input: &[u8], file_data: &mut FileData, editor_state: &mut EditorState) {
    let num = input[0] + input[1] + input[2];
    match num {
        // escape key handler
        27 => editor_state.update_editor_mode(EditorMode::Normal),
        // up arrow or k key
        183 | b'k' => {
            let cursor_coords = Cursor::get_cursor_coords();
            let curr_line = cursor_coords.line as usize;
            let curr_col = cursor_coords.col as usize;
            let pair = file_data
                .file_contents_buffer
                .find_valid_move("up", curr_line, curr_col);
            match pair {
                Some(pair) => cursor::move_cursor_to(pair.0 as u32, pair.1 as u32),
                None => (),
            }
            // if not line 1 locate data where my cursor now is in the
            // data structure
        }
        // down arrow or j key
        184 | b'j' => {
            let cursor_coords = Cursor::get_cursor_coords();
            let curr_line = cursor_coords.line as usize;
            let curr_col = cursor_coords.col as usize;
            let pair = file_data
                .file_contents_buffer
                .find_valid_move("down", curr_line, curr_col);
            match pair {
                Some(pair) => cursor::move_cursor_to(pair.0 as u32, pair.1 as u32),
                None => (),
            }
            // get width of the screen
            // get what column i am in
            //
            //
            // if not last line locate data where my cursor now is in the
            // data structure
        }
        // right arrow or l key
        185 | b'l' => {
            let cursor_coords = Cursor::get_cursor_coords();
            let curr_line = cursor_coords.line as usize;
            let curr_col = cursor_coords.col as usize;
            let pair = file_data
                .file_contents_buffer
                .find_valid_move("right", curr_line, curr_col);
            match pair {
                Some(pair) => cursor::move_cursor_to(pair.0 as u32, pair.1 as u32),
                None => (),
            }
        }
        // left arrow or h key
        186 | b'h' => {
            let cursor_coords = Cursor::get_cursor_coords();
            let curr_line = cursor_coords.line as usize;
            let curr_col = cursor_coords.col as usize;
            let pair = file_data
                .file_contents_buffer
                .find_valid_move("left", curr_line, curr_col);
            match pair {
                Some(pair) => cursor::move_cursor_to(pair.0 as u32, pair.1 as u32),
                None => (),
            }
        }
        _ => (),
    }
}
/// handles input given once user has entered 'command' mode. This mode is entered from normal mode
/// by entering a colon ':' key. Once in this mode the user can enter q to quit the editor or w
/// to write the current file. Entering 'wq' writes the current file before exiting the program.
///
/// function returns Ok with some integer in the case of a valid character. All escape characters
/// for command mode, including <C-c>
fn command_mode_handler(
    input: &[u8],
    editor_state: &mut EditorState,
    file_data: &mut FileData,
    command: &mut String,
) {
    editor_state.previous_mode = EditorMode::Command;
    match input[0] {
        // return/enter key code
        13 => {
            let len = command.len();
            let mut command_chars = command.chars();
            for _ in 0..len {
                command_parser(&command_chars.next(), file_data, editor_state)
            }
            if editor_state.editor_mode != EditorMode::ShutDown {
                editor_state.update_editor_mode(EditorMode::Normal);
            }
            String::clear(command);
        }
        // backspace key code
        127 => {
            command.pop();
            cursor::backspace();
        }
        // code for <C-c> | ascii code for Esc
        3 | 27 => {
            editor_state.update_editor_mode(EditorMode::Normal);
        }
        _ => {
            command.push(input[0] as char);
            cursor::write_char(&input[0]);
        }
    };
}

/// this function handles the parsing of commands recieved from command mode upon recieving input
/// of the Enter key.
fn command_parser(
    command: &Option<char>,
    file_data: &mut FileData,
    editor_state: &mut EditorState,
) {
    match command {
        Some(c) => match c {
            'q' => {
                editor_state.update_editor_mode(EditorMode::ShutDown);
            }
            'w' => file_data.save_file_contents(),
            _ => (),
        },
        None => {
            write!(io::stdout(), "no command given")
                .unwrap_or_else(|e| panic!("io error occurred during write: {e}"));
        }
    }
}

fn graceful_exit(original_settings: &Termios) {
    terminol::disable_alternate_buffer();
    terminol::disable_raw_mode(original_settings);
}
