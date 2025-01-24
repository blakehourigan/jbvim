pub mod config;
pub use config::FileData;
mod tui;
use config::{EditorMode, EditorState};
use gap_buffer::GapBuffer;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::process;
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

    let file_data = FileData::build(cmd_args).unwrap_or_else(|err| {
        println!("problem parsing args: {err}");
        graceful_exit(&original_settings);
        process::exit(1)
    });
    let file_contents = fs::read_to_string(&file_data.file_name).unwrap();
    tui::write_existing_file(&file_contents);
    let mut file_buffer = GapBuffer::new(Some(file_contents));

    cursor::restore_cursor_position();
    io::stdout()
        .flush()
        .unwrap_or_else(|e| panic!("io error occurred during flush: {e}"));

    let mut command = String::new();

    loop {
        let mut input = [0u8; 3];
        if editor_state.editor_mode != EditorMode::ShutDown {
            io::stdin().read(&mut input)?;
        }
        match editor_state.get_current_mode() {
            EditorMode::Normal => normal_mode_handler(&input, &mut editor_state, &mut file_buffer),
            EditorMode::Insert => insert_mode_handler(&input, &mut editor_state, &mut file_buffer),
            EditorMode::Visual => normal_mode_handler(&input, &mut editor_state, &mut file_buffer),
            EditorMode::Command => command_mode_handler(
                &input,
                &mut editor_state,
                &mut file_buffer,
                &mut command,
                &file_data,
            ),
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

fn normal_mode_handler(input: &[u8], editor_state: &mut EditorState, file_buffer: &mut GapBuffer) {
    cursor::enable_standard_cursor();
    match input[0] {
        b':' => {
            editor_state.update_editor_mode(EditorMode::Command);
        }
        // 13 is enter, 127 is backspace, 27 is escape
        b'j' | b'k' | b'l' | b'h' | 13 | 127 | 27 => {
            basic_movement_handler(input, file_buffer, editor_state);
        }
        b'i' => {
            editor_state.update_editor_mode(EditorMode::Insert);
        }
        b'v' => editor_state.update_editor_mode(EditorMode::Visual),
        b'a' => {
            cursor::move_right(1);
            file_buffer.move_cursor_right();
            editor_state.update_editor_mode(EditorMode::Insert);
        }
        b'0' => {
            while !file_buffer.is_line_beginning() {
                move_left_one(file_buffer);
            }
        }
        b'$' => {
            while !file_buffer.next_is_line_end() {
                move_right_one(file_buffer);
            }
        }
        b'w' => {
            while !file_buffer.next_is_space() {
                if file_buffer.next_is_line_end() {
                    break;
                }
                move_right_one(file_buffer);
            }
            while file_buffer.next_is_space() {
                if file_buffer.next_is_line_end() {
                    break;
                }
                move_right_one(file_buffer);
            }
            if !file_buffer.next_is_line_end() {
                move_right_one(file_buffer);
            }
        }
        _ => (),
    }
}

fn insert_mode_handler(input: &[u8], editor_state: &mut EditorState, file_buffer: &mut GapBuffer) {
    match input[0] {
        // return/enter
        13 => {
            file_buffer.insert_left('\n');
            file_buffer.insert_left(' ');
            cursor::return_newline();
        }
        //backspace
        127 => {
            file_buffer.delete_char();
            cursor::backspace();
            // grab from 1 after the gaps end until the next newline or until the eof
            let line_end = file_buffer.grab_to_line_end();
            // tell tui that we need to update the line we're on with the content we just got
            // after our current position
            cursor::write_char(&input[0]);
            tui::update_line(line_end, true);
        }
        // 3: <C-c> | 27: Esc
        3 | 27 => basic_movement_handler(input, file_buffer, editor_state),
        // any other character just insert it in
        _ => {
            if file_buffer.next_is_line_end() {
                file_buffer.insert_left(input[0] as char);
                cursor::write_char(&input[0]);
            } else {
                // insert the char
                file_buffer.insert_left(input[0] as char);
                // grab from 1 after the gaps end until the next newline or until the eof
                let line_end = file_buffer.grab_to_line_end();
                // tell tui that we need to update the line we're on with the content we just got
                // after our current position
                cursor::write_char(&input[0]);
                tui::update_line(line_end, false);
            }
        }
    };
}

fn basic_movement_handler(
    input: &[u8],
    file_buffer: &mut GapBuffer,
    editor_state: &mut EditorState,
) {
    let num = input[0] + input[1] + input[2];
    match num {
        // 3: <C-c> | 27: escape key handler
        3 | 27 => editor_state.update_editor_mode(EditorMode::Normal),
        // up arrow or k key
        183 | b'k' => {
            let cursor_coords = Cursor::get_cursor_coords();
            let curr_line = cursor_coords.line;
            let curr_col = cursor_coords.col;

            if file_buffer.is_first_line() {
                return;
            }

            find_previous_newline(file_buffer);
            file_buffer.move_cursor_left();
            cursor::move_left(1);

            find_previous_newline(file_buffer);
            file_buffer.move_cursor_left();

            cursor::move_cursor_to(curr_line - 1, 1);

            for _ in 1..curr_col {
                if file_buffer.next_is_line_end() {
                    break;
                }
                file_buffer.move_cursor_right();
                cursor::move_right(1);
            }
        }
        // down arrow or j key
        184 | b'j' | 13 => {
            let cursor_coords = Cursor::get_cursor_coords();
            let curr_line = cursor_coords.line;
            let curr_col = cursor_coords.col;

            if file_buffer.is_last_line() {
                return;
            }

            find_next_newline(file_buffer);

            file_buffer.move_cursor_right();
            cursor::move_cursor_to(curr_line + 1, 1);

            for _ in 1..curr_col {
                if file_buffer.next_is_line_end() {
                    break;
                }
                file_buffer.move_cursor_right();
                cursor::move_right(1);
            }
        }
        // right arrow or l key
        185 | b'l' => {
            if !file_buffer.next_is_line_end() {
                file_buffer.move_cursor_right();
                cursor::move_right(1);
            }
        }
        // left arrow or h key
        186 | b'h' | 127 => {
            if !file_buffer.is_line_beginning() {
                file_buffer.move_cursor_left();
                cursor::move_left(1);
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
    file_buffer: &mut GapBuffer,
    command: &mut String,
    file_data: &FileData,
) {
    editor_state.previous_mode = EditorMode::Command;
    match input[0] {
        // return/enter key code
        13 => {
            let len = command.len();
            let mut command_chars = command.chars();
            for _ in 0..len {
                command_parser(&command_chars.next(), file_buffer, file_data, editor_state)
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
    file_buffer: &mut GapBuffer,
    file_data: &FileData,
    editor_state: &mut EditorState,
) {
    match command {
        Some(c) => match c {
            'q' => {
                editor_state.update_editor_mode(EditorMode::ShutDown);
            }
            'w' => save_file_contents(file_data, file_buffer),
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
fn find_next_newline(file_buffer: &mut GapBuffer) {
    while !file_buffer.cursor_is_newline() {
        move_right_one(file_buffer);
    }
}
fn find_previous_newline(file_buffer: &mut GapBuffer) {
    while !file_buffer.before_cursor_is_newline() {
        move_left_one(file_buffer);
    }
}
fn move_right_one(file_buffer: &mut GapBuffer) {
    file_buffer.move_cursor_right();
    cursor::move_right(1);
}
fn move_left_one(file_buffer: &mut GapBuffer) {
    file_buffer.move_cursor_left();
    cursor::move_left(1);
}

fn save_file_contents(file_data: &FileData, file_buffer: &mut GapBuffer) {
    let data = file_buffer.get_content();

    fs::write(format!("./{}", file_data.file_name), data).expect("should write to /file_name");
}
