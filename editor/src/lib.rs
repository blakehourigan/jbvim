mod tui;
use gap_buffer::GapBuffer;
use libc;
use std::error::Error;
use std::io::{self, Read, Write};
use std::process;
use termux;

#[derive(Clone, Copy)]
enum EditorMode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl EditorMode {
    fn value(&self) -> String {
        match *self {
            EditorMode::Normal => String::from("normal"),
            EditorMode::Insert => String::from("insert"),
            EditorMode::Visual => String::from("visual"),
            EditorMode::Command => String::from("command"),
        }
    }
}

struct EditorData {
    reader: io::Stdin,
    character_buffer: [u8; 1],
    file_contents_buffer: GapBuffer,
    editor_mode: EditorMode,
    previous_mode: EditorMode,
    cursor: termux::cursor::Cursor,
    terminal_attributes: libc::winsize,
}
impl EditorData {
    fn update_editor_mode(&mut self, mode: EditorMode) {
        self.previous_mode = self.editor_mode;
        self.editor_mode = mode;
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    // fills array with 0 once
    let buffer = [0; 1];
    let file_contents = GapBuffer::new(Option::None);
    let editor_mode = EditorMode::Normal;
    let previous_mode = EditorMode::Normal;
    let cursor = termux::cursor::Cursor::new();
    let reader = io::stdin();
    let terminal_window_attr = termux::get_terminal_size();

    let mut editor_data = EditorData {
        reader,
        character_buffer: buffer,
        file_contents_buffer: file_contents,
        editor_mode,
        previous_mode,
        cursor,
        terminal_attributes: terminal_window_attr,
    };

    termux::enable_alternate_buffer()?;
    termux::enable_raw_mode()?;
    termux::clear_screen()?;

    editor_data.cursor.move_home()?;

    tui::update_tui(&mut editor_data)?;
    io::stdout().flush().unwrap();

    let mut command = String::new();
    loop {
        // opening reader gets rid of the shell prompt guy
        editor_data
            .reader
            .read_exact(&mut editor_data.character_buffer)?;

        match editor_data.editor_mode {
            EditorMode::Normal => normal_mode_handler(&mut editor_data)?,
            EditorMode::Insert => insert_mode_handler(&mut editor_data)?,
            EditorMode::Visual => normal_mode_handler(&mut editor_data)?,
            EditorMode::Command => command_mode_handler(&mut editor_data, &mut command)?,
        };
        tui::update_tui(&mut editor_data)?;
        io::stdout().flush().unwrap();
    }
}

fn normal_mode_handler(editor_data: &mut EditorData) -> Result<(), Box<dyn Error>> {
    match editor_data.character_buffer[0] {
        b':' => {
            editor_data.update_editor_mode(EditorMode::Command);
        }

        b'j' => {
            editor_data.cursor.move_down(1)?;
            ()
        }
        b'k' => {
            editor_data.cursor.move_up(1)?;
            ()
        }
        b'l' => {
            editor_data.cursor.move_right(1)?;
            editor_data.file_contents_buffer.move_cursor_right()
        }
        b'h' => {
            editor_data.cursor.move_left(1, false)?;
            editor_data.file_contents_buffer.move_cursor_left()
        }

        b'i' => {
            editor_data.update_editor_mode(EditorMode::Insert);
        }
        b'v' => editor_data.update_editor_mode(EditorMode::Visual),
        // return/enter
        13 => {
            editor_data.cursor.move_down(1)?;
            ()
        }
        // spacebar... may or may not re-add later
        //32 => {
        //    editor_data.cursor.move_right(1)?;
        //    ()
        //}
        //backspace
        127 => {
            editor_data.cursor.move_left(1, false)?;
        }
        27 => (),
        _ => (),
    };
    Ok(())
}
/// this function handles the parsing of commands recieved from command mode upon recieving input
/// of the Enter key.
fn command_parser(command: &Option<char>) -> Result<(), Box<dyn Error>> {
    match command {
        Some(c) => match c {
            'q' => {
                termux::disable_alternate_buffer()?;
                process::exit(0);
            }
            'w' => (),
            _ => (),
        },
        None => {
            write!(io::stdout(), "no command given")?;
            ()
        }
    }
    Ok(())
}

/// handles input given once user has entered 'command' mode. This mode is entered from normal mode
/// by entering a colon ':' key. Once in this mode the user can enter q to quit the editor or w
/// to write the current file. Entering 'wq' writes the current file before exiting the program.
///
/// function returns Ok with some integer in the case of a valid character. All escape characters
/// for command mode, including <C-c>
fn command_mode_handler(
    editor_data: &mut EditorData,
    command: &mut String,
) -> Result<(), Box<dyn Error>> {
    editor_data.previous_mode = EditorMode::Command;
    match editor_data.character_buffer[0] {
        // return/enter key code
        13 => {
            let len = command.len();
            let mut command_chars = command.chars();
            for _ in 0..len {
                match command_parser(&command_chars.next()) {
                    Ok(_) => (),
                    Err(_) => {
                        write!(io::stdout(), "command not found")?;
                        break;
                    }
                }
            }
            editor_data.update_editor_mode(EditorMode::Normal);
            String::clear(command);
            return Ok(());
        }
        // backspace key code
        127 => {
            command.pop();
            editor_data.cursor.backspace(true)?;
            ()
        }
        // code for <C-c>
        3 => {
            editor_data.update_editor_mode(EditorMode::Normal);
            editor_data.cursor.restore_cursor_position()?;
            io::stdout().flush().unwrap();
        }
        // ascii code for Esc
        27 => {
            editor_data.update_editor_mode(EditorMode::Normal);
            editor_data.cursor.restore_cursor_position()?;
            io::stdout().flush().unwrap();
        }
        _ => {
            command.push(editor_data.character_buffer[0] as char);
            editor_data
                .cursor
                .write_char(&editor_data.character_buffer[0], true)?;
        }
    };
    Ok(())
}

fn insert_mode_handler(editor_data: &mut EditorData) -> Result<(), Box<dyn Error>> {
    match editor_data.character_buffer[0] {
        // return/enter
        13 => {
            editor_data.cursor.move_down(1)?;
        }
        //backspace
        127 => {
            editor_data.file_contents_buffer.delete_char();
            editor_data.cursor.backspace(false)?;
            ()
        }
        // <C-c>
        3 => {
            editor_data.update_editor_mode(EditorMode::Normal);
        }
        // Esc
        27 => {
            editor_data.update_editor_mode(EditorMode::Normal);
        }
        _ => {
            editor_data
                .file_contents_buffer
                .insert_left(editor_data.character_buffer[0] as char);

            editor_data
                .cursor
                .write_char(&editor_data.character_buffer[0], false)?;
        }
    };
    Ok(())
}
