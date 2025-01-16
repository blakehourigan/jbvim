pub mod config;
mod tui;
pub use config::FileData;
use config::{EditorMode, EditorState};
use std::env;
use std::error::Error;
use std::io::{self, Read, Write};
use std::process;
use terminol::cursor;
use termios::Termios;

pub fn run(cmd_args: env::Args) -> Result<Termios, Box<dyn Error>> {
    let mut editor_state = EditorState::new(EditorMode::Normal, EditorMode::Normal);
    let original_settings = terminol::enable_raw_mode();

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

        b'j' => {
            cursor::move_down(1);
        }
        b'k' => {
            cursor::move_up(1);
        }
        b'l' => {
            cursor::move_right(1);
            file_data.file_contents_buffer.move_cursor_right()
        }
        b'h' => {
            cursor::move_left(1);
            file_data.file_contents_buffer.move_cursor_left()
        }

        b'i' => {
            editor_state.update_editor_mode(EditorMode::Insert);
            cursor::enable_bar_cursor();
        }
        b'v' => editor_state.update_editor_mode(EditorMode::Visual),
        // return/enter
        13 => cursor::move_down(1),
        //backspace
        127 => cursor::move_left(1),
        27 => {
            let num = input[0] + input[1] + input[2];
            match num {
                27 => editor_state.update_editor_mode(EditorMode::Normal),
                // up arrow key
                183 => {
                    cursor::move_up(1);
                }
                // down arrow key
                184 => {
                    cursor::move_down(1);
                }
                // right arrow key
                185 => {
                    cursor::move_right(1);
                    file_data.file_contents_buffer.move_cursor_right();
                }
                // left arrow key
                186 => {
                    cursor::move_left(1);
                    file_data.file_contents_buffer.move_cursor_left();
                }
                _ => (),
            }
        }
        _ => (),
    };
}

fn insert_mode_handler(input: &[u8], editor_state: &mut EditorState, file_data: &mut FileData) {
    match input[0] {
        // return/enter
        13 => {
            file_data.file_contents_buffer.insert_left('\r');
            file_data.file_contents_buffer.insert_left('\n');
            cursor::return_newline();
        }
        //backspace
        127 => {
            file_data.file_contents_buffer.delete_char();
            cursor::backspace();
        }
        // <C-c> | Esc
        3 => {
            editor_state.update_editor_mode(EditorMode::Normal);
        }
        27 => {
            let num = input[0] + input[1] + input[2];
            match num {
                27 => editor_state.update_editor_mode(EditorMode::Normal),
                // up arrow key
                183 => {
                    cursor::move_up(1);
                }
                // down arrow key
                184 => {
                    cursor::move_down(1);
                }
                // right arrow key
                185 => {
                    cursor::move_right(1);
                    file_data.file_contents_buffer.move_cursor_right();
                }
                // left arrow key
                186 => {
                    cursor::move_left(1);
                    file_data.file_contents_buffer.move_cursor_left();
                }
                _ => (),
            }
        }
        _ => {
            //if file_data.file_contents_buffer.is_line_end() {
            //    file_data.file_contents_buffer.insert_left(input[0] as char);
            //    cursor::write_char(&input[0]);
            //} else {
            //    // insert the char
            //    file_data.file_contents_buffer.insert_left(input[0] as char);
            //    // grab from 1 after the gaps end until the next newline or until the eof
            //    let line_end = file_data.file_contents_buffer.grab_to_line_end();
            //    // tell tui that we need to update the line we're on with the content we just got
            //    // after our current position
            //    tui::update_line();
            //}
        }
    };
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
