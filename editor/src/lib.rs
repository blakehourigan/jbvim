mod tui;
use gap_buffer::GapBuffer;
use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::process;
use terminol::cursor;
use termios::Termios;

#[derive(Clone, Copy, PartialEq)]
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

pub struct FileData {
    file_name: String,
    file_handle: fs::File,
    file_contents_buffer: GapBuffer,
}

impl FileData {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<FileData, &'static str> {
        args.next();

        let file_name = match args.next() {
            Some(arg) => arg,
            None => return Err("No file name provided, exiting..."),
        };

        let file_handle;
        let file_contents;
        //if Path::new(&file_name).exists() {
        // open the file and read the lines into the terminal... or
        // at least collect it so that it is easy to do so later.
        file_handle = fs::File::open(&file_name).unwrap();
        file_contents = fs::read_to_string(&file_name).unwrap();
        //}
        let file_contents_buffer = GapBuffer::new(Some(file_contents));

        Ok(FileData {
            file_name,
            file_handle,
            file_contents_buffer,
        })
    }
}
struct EditorState {
    editor_mode: EditorMode,
    previous_mode: EditorMode,
    original_settings: Option<Termios>,
}

impl EditorState {
    fn new(editor_mode: EditorMode, previous_mode: EditorMode) -> Self {
        EditorState {
            editor_mode,
            previous_mode,
            original_settings: Option::None,
        }
    }
    fn update_editor_mode(&mut self, mode: EditorMode) {
        self.previous_mode = self.editor_mode;
        self.editor_mode = mode;
    }
}

pub fn run(file_data: &mut FileData) -> Result<(), Box<dyn Error>> {
    let mut editor_state = EditorState::new(EditorMode::Normal, EditorMode::Normal);

    terminol::enable_alternate_buffer()?;
    editor_state.original_settings = Some(terminol::enable_raw_mode()?);
    terminol::clear_screen()?;

    cursor::move_home()?;

    tui::update_tui(&mut editor_state)?;
    io::stdout().flush().unwrap();

    let mut command = String::new();
    loop {
        let mut input = [0u8; 1];
        // opening reader gets rid of the shell prompt guy
        io::stdin().read_exact(&mut input)?;

        match editor_state.editor_mode {
            EditorMode::Normal => normal_mode_handler(&input, &mut editor_state, file_data)?,
            EditorMode::Insert => insert_mode_handler(&input, file_data, &mut editor_state)?,
            EditorMode::Visual => normal_mode_handler(&input, &mut editor_state, file_data)?,
            EditorMode::Command => {
                command_mode_handler(&input, &mut editor_state, file_data, &mut command)?
            }
        };
        tui::update_tui(&mut editor_state)?;
        io::stdout().flush().unwrap();
    }
}

fn normal_mode_handler(
    input: &[u8],
    editor_state: &mut EditorState,
    file_data: &mut FileData,
) -> Result<(), Box<dyn Error>> {
    match input[0] {
        b':' => {
            editor_state.update_editor_mode(EditorMode::Command);
        }

        b'j' => {
            cursor::move_down(1)?;
            ()
        }
        b'k' => {
            cursor::move_up(1)?;
            ()
        }
        b'l' => {
            cursor::move_right(1)?;
            file_data.file_contents_buffer.move_cursor_right()
        }
        b'h' => {
            cursor::move_left(0)?;
            file_data.file_contents_buffer.move_cursor_left()
        }

        b'i' => {
            editor_state.update_editor_mode(EditorMode::Insert);
            cursor::enable_bar_cursor()?;
        }
        b'v' => editor_state.update_editor_mode(EditorMode::Visual),
        // return/enter
        13 => cursor::move_down(1)?,
        //backspace
        127 => cursor::move_left(1)?,
        27 => (),
        _ => (),
    };
    Ok(())
}

fn insert_mode_handler(
    input: &[u8],
    file_data: &mut FileData,
    editor_state: &mut EditorState,
) -> Result<(), Box<dyn Error>> {
    match input[0] {
        // return/enter
        13 => {
            file_data.file_contents_buffer.insert_left('\n');
            cursor::return_newline()?;
        }
        //backspace
        127 => {
            file_data.file_contents_buffer.delete_char();
            cursor::backspace()?;
        }
        // <C-c> | Esc
        3 | 27 => {
            editor_state.update_editor_mode(EditorMode::Normal);
        }
        _ => {
            file_data.file_contents_buffer.insert_left(input[0] as char);

            cursor::write_char(&input[0])?;
        }
    };
    Ok(())
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
) -> Result<(), Box<dyn Error>> {
    editor_state.previous_mode = EditorMode::Command;
    match input[0] {
        // return/enter key code
        13 => {
            let len = command.len();
            let mut command_chars = command.chars();
            for _ in 0..len {
                match command_parser(&command_chars.next(), file_data, editor_state) {
                    Err(_) => {
                        write!(io::stdout(), "command not found")?;
                        break;
                    }
                    _ => (),
                }
            }

            editor_state.update_editor_mode(EditorMode::Normal);
            String::clear(command);
            return Ok(());
        }
        // backspace key code
        127 => {
            command.pop();
            cursor::backspace()?;
        }
        // code for <C-c> | ascii code for Esc
        3 | 27 => {
            editor_state.update_editor_mode(EditorMode::Normal);
            io::stdout().flush().unwrap();
        }
        _ => {
            command.push(input[0] as char);
            cursor::write_char(&input[0])?;
        }
    };
    Ok(())
}

/// this function handles the parsing of commands recieved from command mode upon recieving input
/// of the Enter key.
fn command_parser(
    command: &Option<char>,
    file_data: &mut FileData,
    editor_state: &mut EditorState,
) -> Result<(), Box<dyn Error>> {
    match command {
        Some(c) => match c {
            'q' => {
                terminol::disable_alternate_buffer()?;
                terminol::disable_raw_mode(&editor_state.original_settings.unwrap())?;
                process::exit(0);
            }
            'w' => save_file_contents(
                file_data.file_name.clone(),
                file_data.file_contents_buffer.get_content(),
            ),
            _ => (),
        },
        None => {
            write!(io::stdout(), "no command given")?;
        }
    }
    Ok(())
}

fn save_file_contents(file_name: String, file_content: String) {
    let data = file_content;

    fs::write(format!("./{file_name}"), data).expect("should write to /file_name");
}
