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
fn load_exiting_file_content(file_contents: String) -> GapBuffer<GapBuffer<char>> {
    let mut content_buffer = GapBuffer::new();
    for line in file_contents.lines() {
        let mut line_buf = GapBuffer::new();

        for c in line.chars() {
            line_buf.insert_left(c);
        }
        line_buf.insert_left('\n');
        line_buf.reset();
        content_buffer.insert_left(line_buf);
    }
    content_buffer.reset();
    content_buffer
}
fn setup_terminal(cmd_args: env::Args) -> EditorConfig {
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
        //graceful_exit(&original_settings);
        process::exit(1)
    });
    let file_contents = fs::read_to_string(&file_data.file_name).unwrap();

    let mut content_buffer = load_exiting_file_content(file_contents);

    //print!("{:?}", &content_buffer);
    tui::write_existing_file(&content_buffer.get_content());

    cursor::restore_cursor_position();
    io::stdout()
        .flush()
        .unwrap_or_else(|e| panic!("io error occurred during flush: {e}"));

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

        io::stdout()
            .flush()
            .unwrap_or_else(|e| panic!("io error occurred during flush: {e}"));
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
            cursor::move_right(1);
            content_buffer.move_gap_right();
            editor_state.update_editor_mode(EditorMode::Insert);
        }
        b'0' => {
            line_buf.reset();
            cursor::move_cursor_to(line, 1);
        }

        b'$' => {
            let len = line_buf.buffer.len();
            let mut new_col = 0;
            for _ in 0..len {
                if !line_buf.next_is_empty() {
                    line_buf.move_gap_right();
                    new_col += 1;
                } else {
                    break;
                }
            }
            cursor::move_cursor_to(line, new_col);
        }
        _ => (),
    }
}

fn insert_mode_handler(
    input: &[u8],
    editor_state: &mut EditorState,
    content_buffer: &mut GapBuffer<GapBuffer<char>>,
) {
    let line_buf = content_buffer.get_nested();
    match input[0] {
        // return/enter
        13 => {
            line_buf.insert_left('\n');
            cursor::return_newline();
        }
        //backspace
        127 => {
            line_buf.delete_item();
            cursor::backspace();
            let line_end = line_buf.grab_to_end();
            // tell tui that we need to update the line we're on with the content we just got
            // after our current position
            cursor::write_char(&input[0]);
            tui::update_line(line_end, true);
        }
        // <C-c> | Esc
        3 | 27 | 183 | 184 | 185 | 186 => {
            basic_movement_handler(input, content_buffer, editor_state);
        }
        _ => {
            if line_buf.is_line_end() {
                line_buf.insert_left(input[0] as char);
                cursor::write_char(&input[0]);
            } else {
                // insert the char
                line_buf.insert_left(input[0] as char);
                // grab from 1 after the gaps end until the next newline or until the eof
                let line_end = line_buf.grab_to_end();
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
    content_buffer: &mut GapBuffer<GapBuffer<char>>,
    editor_state: &mut EditorState,
) {
    let num = input[0] + input[1] + input[2];

    match num {
        // escape key handler
        3 | 27 => editor_state.update_editor_mode(EditorMode::Normal),
        // up arrow or k key
        183 | b'k' => {
            if !content_buffer.last_is_first() {
                content_buffer.move_gap_left();
                let line = Cursor::get_cursor_coords().line - 1;
                let col = Cursor::get_cursor_coords().col;
                // get new line
                let line_buf = content_buffer.get_nested();

                let mut new_col = 0;
                for _ in 0..col {
                    if !line_buf.next_is_empty() {
                        line_buf.move_gap_right();
                        new_col += 1;
                    } else {
                        break;
                    }
                }
                cursor::move_cursor_to(line, new_col);
            }
        }
        // down arrow or j key
        184 | b'j' => {
            if !content_buffer.next_is_empty() {
                content_buffer.move_gap_right();
                let line = Cursor::get_cursor_coords().line + 1;
                let col = Cursor::get_cursor_coords().col;

                let line_buf = content_buffer.get_nested();
                let mut new_col = 0;
                for _ in 0..col {
                    if !line_buf.next_is_empty() {
                        line_buf.move_gap_right();
                        new_col += 1;
                    } else {
                        break;
                    }
                }
                cursor::move_cursor_to(line, new_col);
            }
        }
        // right arrow or l key
        185 | b'l' => {
            let line = Cursor::get_cursor_coords().line + 1;
            let col = Cursor::get_cursor_coords().col;

            let line_buf = content_buffer.get_nested();
            if !line_buf.is_line_end() {
                line_buf.move_gap_right();
                cursor::move_right(1);
            }
        }
        // left arrow or h key
        186 | b'h' => {
            let line = Cursor::get_cursor_coords().line + 1;
            let col = Cursor::get_cursor_coords().col;

            let line_buf = content_buffer.get_nested();
            if !line_buf.is_line_begin() {
                line_buf.move_gap_left();
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
