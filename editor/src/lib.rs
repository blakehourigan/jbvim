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

struct EditorConfig {
    editor_state: EditorState,
    original_settings: Termios,
    file_data: FileData,
    gap_buffer: GapBuffer<GapBuffer<char>>,
}

fn initialize_tui_state() {
    cursor::move_home();
    cursor::save_cursor_position();
    terminol::enable_alternate_buffer();
    terminol::clear_screen();
}
fn setup_terminal(cmd_args: env::Args) -> EditorConfig {
    let mut editor_state = EditorState::new(EditorMode::Normal, EditorMode::Normal);
    let original_settings = terminol::enable_raw_mode();

    initialize_tui_state();

    let file_data = FileData::build(cmd_args).unwrap_or_else(|err| {
        println!("problem parsing args: {err}");
        //graceful_exit(&original_settings);
        panic!("broken")
    });

    let file_contents = fs::read_to_string(&file_data.file_name).unwrap();

    let max_size = terminol::get_terminal_size().ws_col as usize - 50;
    let content_buffer = GapBuffer::build_nested(&file_contents, max_size);

    let content = content_buffer.get_content();
    tui::write_existing_file(content);

    cursor::restore_cursor_position();
    tui::update_tui(&mut editor_state);

    EditorConfig {
        editor_state,
        original_settings,
        file_data,
        gap_buffer: content_buffer,
    }
}
pub fn run(cmd_args: env::Args) -> Result<Termios, Box<dyn Error>> {
    let mut editor_config = setup_terminal(cmd_args);

    let mut command = String::new();

    loop {
        let mut input = [0u8; 3];
        // opening reader gets rid of the shell prompt guy
        if editor_config.editor_state.editor_mode != EditorMode::ShutDown {
            io::stdin().read(&mut input)?;
        }
        match editor_config.editor_state.get_current_mode() {
            EditorMode::Normal => normal_mode_handler(
                &input,
                &mut editor_config.editor_state,
                &mut editor_config.gap_buffer,
            ),
            EditorMode::Insert => insert_mode_handler(
                &input,
                &mut editor_config.editor_state,
                &mut editor_config.gap_buffer,
            ),
            EditorMode::Visual => normal_mode_handler(
                &input,
                &mut editor_config.editor_state,
                &mut editor_config.gap_buffer,
            ),
            EditorMode::Command => command_mode_handler(
                &input,
                &mut editor_config.editor_state,
                &mut editor_config.gap_buffer,
                &mut editor_config.file_data,
                &mut command,
            ),
            EditorMode::ShutDown => {
                graceful_exit(&editor_config.original_settings);
                break;
            }
        };
        tui::update_tui(&mut editor_config.editor_state);
    }
    Ok(editor_config.original_settings)
}

fn normal_mode_handler(
    input: &[u8],
    editor_state: &mut EditorState,
    content_buffer: &mut GapBuffer<GapBuffer<char>>,
) {
    cursor::enable_standard_cursor();
    let line = Cursor::get_cursor_coords().line;
    let line_buf = content_buffer.get_nested();
    match input[0] {
        b':' => {
            editor_state.update_editor_mode(EditorMode::Command);
        }
        // enter | escape | arrow keys| <C-c>
        b'j' | b'k' | b'l' | b'h' | 13 | 27 | 183 | 184 | 185 | 186 | 3 => {
            basic_movement_handler(input, content_buffer, editor_state);
        }
        b'i' => {
            editor_state.update_editor_mode(EditorMode::Insert);
        }
        b'v' => editor_state.update_editor_mode(EditorMode::Visual),
        b'a' => {
            move_right(line_buf);
            editor_state.update_editor_mode(EditorMode::Insert);
        }
        b'0' => {
            line_buf.reset();
            cursor::move_cursor_to(line, 1);
        }

        b'$' => {
            let line = Cursor::get_cursor_coords().line;
            let len = line_buf.grab_to_end(true).len();

            line_buf.move_to_last_char();
            cursor::move_cursor_to(line, len);
        }
        b'w' => {
            let i = line_buf.move_to_next_word();
            for _ in 0..i {
                cursor::move_right(1);
            }
        }
        _ => (),
    }
}

fn insert_mode_handler(
    input: &[u8],
    editor_state: &mut EditorState,
    content_buffer: &mut GapBuffer<GapBuffer<char>>,
) {
    match input[0] {
        // return/enter
        13 => {
            let line = Cursor::get_cursor_coords().line;
            content_buffer.move_line_contents_enter(line);
            terminol::clear_end_of_screen();
        }
        //backspace
        127 => {
            // if at beginning of line then move the lines contents up to the last line
            // only if it does not exceed the limit for length of the terminal window
            let line_buf = content_buffer.get_nested();
            terminol::clear_end_of_line();

            if line_buf.is_buf_begin() {
                let line = Cursor::get_cursor_coords().line;
                content_buffer.move_line_contents_backspace(line, line - 1);
            } else {
                line_buf.delete_item();
                cursor::backspace();
                let line_end = line_buf.grab_to_end(false);
                // tell tui that we need to update the line we're on with the content we just got
                // after our current position
                cursor::write_char(&input[0]);
                tui::update_line(line_end);
            }
        }
        // <C-c> | Esc
        3 | 27 | 183 | 184 | 185 | 186 => {
            basic_movement_handler(input, content_buffer, editor_state);
        }
        _ => {
            let line_buf = content_buffer.get_nested();
            if line_buf.is_line_end() {
                line_buf.insert_left(input[0] as char);
                cursor::write_char(&input[0]);
            } else {
                // insert the char
                terminol::clear_end_of_line();
                // grab from 1 after the gaps end until the next newline or until the eof
                let line_end = line_buf.grab_to_end(false);
                line_buf.insert_left(input[0] as char);
                cursor::write_char(&input[0]);
                // tell tui that we need to update the line we're on with the content we just got
                // after our current position
                tui::update_line(line_end);
            }
        }
    };
}

fn basic_movement_handler(
    input: &[u8],
    content_buffer: &mut GapBuffer<GapBuffer<char>>,
    editor_state: &mut EditorState,
) {
    let num = input[0] + input[1] + input[2];

    match num {
        // escape key handler
        3 | 27 => editor_state.update_editor_mode(EditorMode::Normal),
        // up arrow or k key
        183 | b'k' => {
            if content_buffer.is_first_line() {
                return;
            } else {
                content_buffer.move_gap_left();
                let line_buf = content_buffer.get_nested();
                let line_len = line_buf.get_len();

                line_buf.reset();

                let line = Cursor::get_cursor_coords().line;
                let new_line = line - 1;

                let col = Cursor::get_cursor_coords().col;

                let new_col;

                if line_len >= col {
                    new_col = col;
                } else {
                    new_col = line_len;
                }

                for _ in 1..new_col {
                    line_buf.move_gap_right();
                }

                cursor::move_cursor_to(new_line, new_col);
            }
        }
        // down arrow or j key
        184 | b'j' => {
            content_buffer.move_gap_right();
            if content_buffer.is_last_line() {
                content_buffer.move_gap_left();
                return;
            } else {
                let line_buf = content_buffer.get_nested();
                let line_len = line_buf.get_len();

                line_buf.reset();

                let line = Cursor::get_cursor_coords().line;
                let new_line = line + 1;

                let col = Cursor::get_cursor_coords().col;

                let new_col;

                if line_len >= col {
                    new_col = col;
                } else {
                    new_col = line_len;
                }

                for _ in 0..(new_col - 1) {
                    line_buf.move_gap_right();
                }

                cursor::move_cursor_to(new_line, new_col);
            }
        }
        // right arrow or l key
        185 | b'l' => {
            let line_buf = content_buffer.get_nested();
            if !line_buf.is_line_end() {
                move_right(line_buf);
            }
        }
        // left arrow or h key
        186 | b'h' => {
            let line_buf = content_buffer.get_nested();
            if line_buf.is_buf_begin() {
                return;
            }
            move_left(line_buf);
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
    content_buffer: &mut GapBuffer<GapBuffer<char>>,
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
                command_parser(
                    &command_chars.next(),
                    content_buffer,
                    file_data,
                    editor_state,
                )
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
    content_buffer: &mut GapBuffer<GapBuffer<char>>,
    file_data: &FileData,
    editor_state: &mut EditorState,
) {
    match command {
        Some(c) => match c {
            'q' => {
                editor_state.update_editor_mode(EditorMode::ShutDown);
            }
            'w' => save_file_contents(file_data, content_buffer),
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
pub fn save_file_contents(file_data: &FileData, content_buffer: &mut GapBuffer<GapBuffer<char>>) {
    let data = content_buffer.get_content();

    fs::write(format!("./{}", file_data.file_name), data).expect("should write to /file_name");
}

fn move_right(line_buf: &mut GapBuffer<char>) {
    line_buf.move_gap_right();
    cursor::move_right(1);
}
fn move_left(line_buf: &mut GapBuffer<char>) {
    line_buf.move_gap_left();
    cursor::move_left(1);
}
