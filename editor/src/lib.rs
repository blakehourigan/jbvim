mod tui;
use libc;
use std::error::Error;
use std::io::{self, Read, Write};
use std::process;
use termux;

enum EditorMode {
    Normal,
    Insert,
    Visual,
}

struct EditorData {
    reader: io::Stdin,
    character_buffer: [u8; 1],
    file_contents_buffer: Vec<char>,
    editor_mode: EditorMode,
    cursor: termux::cursor::Cursor,
    terminal_attributes: libc::winsize,
}

pub fn run() -> Result<(), Box<dyn Error>> {
    // fills array with 0 once
    let buffer = [0; 1];
    let file_contents: Vec<char> = vec![];
    let editor_mode = EditorMode::Normal;
    let cursor = termux::cursor::Cursor::new();
    let reader = io::stdin();
    let terminal_window_attr = termux::get_terminal_size();

    let mut editor_data = EditorData {
        reader,
        character_buffer: buffer,
        file_contents_buffer: file_contents,
        editor_mode,
        cursor,
        terminal_attributes: terminal_window_attr,
    };

    termux::enable_alternate_buffer()?;
    termux::enable_raw_mode()?;
    termux::clear_screen()?;

    editor_data.cursor.move_home()?;

    tui::draw_info_tui(&mut editor_data)?;
    io::stdout().flush().unwrap();

    loop {
        // opening reader gets rid of the shell prompt guy
        editor_data
            .reader
            .read_exact(&mut editor_data.character_buffer)?;

        match editor_data.editor_mode {
            EditorMode::Normal => normal_mode_handler(&mut editor_data)?,
            EditorMode::Insert => insert_mode_handler(&mut editor_data)?,
            EditorMode::Visual => normal_mode_handler(&mut editor_data)?,
        };
        tui::draw_info_tui(&mut editor_data)?;
        io::stdout().flush().unwrap();
    }
}

fn normal_mode_handler(editor_data: &mut EditorData) -> Result<Option<u32>, Box<dyn Error>> {
    match editor_data.character_buffer[0] {
        b':' => {
            tui::enable_command_field(editor_data)?;
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
        }
        b'h' => {
            editor_data.cursor.move_left(1)?;
            ()
        }

        b'i' => editor_data.editor_mode = EditorMode::Insert,
        b'v' => editor_data.editor_mode = EditorMode::Visual,
        13 => {
            editor_data.cursor.move_down(1)?;
            ()
        }
        32 => {
            editor_data.cursor.move_right(1)?;
            ()
        }
        127 => {
            editor_data.cursor.move_left(1)?;
        } //backspace
        27 => return Ok(Option::None),
        _ => (),
    };
    Ok(Some(1))
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
) -> Result<Option<u32>, Box<dyn Error>> {
    match editor_data.character_buffer[0] {
        // return/enter key code
        13 => {
            for _ in command.chars() {
                command_parser(&command.chars().next())?;
            }
            return Ok(Option::None);
        }
        // backspace key code
        127 => {
            command.pop();
            editor_data.cursor.backspace()?;
            ()
        }
        // code for <C-c>
        3 => {
            return Ok(Option::None);
        }
        // ascii code for Esc
        27 => return Ok(Option::None),
        _ => {
            command.push(editor_data.character_buffer[0] as char);
            editor_data
                .cursor
                .write_char(&editor_data.character_buffer[0])?;
        }
    };
    Ok(Some(0))
}

fn insert_mode_handler(editor_data: &mut EditorData) -> Result<Option<u32>, Box<dyn Error>> {
    match editor_data.character_buffer[0] {
        // return/enter
        13 => {
            write!(io::stdout(), "\r\n")?;
            return Ok(Option::None);
        }
        //backspace
        127 => {
            editor_data.cursor.backspace()?;
            ()
        }
        3 => {
            editor_data.editor_mode = EditorMode::Normal;
            return Ok(Option::None);
        }
        27 => return Ok(Option::None),
        _ => {
            editor_data
                .file_contents_buffer
                .push(editor_data.character_buffer[0] as char);
            editor_data
                .cursor
                .write_char(&editor_data.character_buffer[0])?;
            editor_data.cursor.move_right(1)?;
        }
    };
    Ok(Some(0))
}
