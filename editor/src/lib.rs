mod tui;
use gap_buffer::GapBuffer;
use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::process;

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
}

impl EditorState {
    fn new(editor_mode: EditorMode, previous_mode: EditorMode) -> Self {
        EditorState {
            editor_mode,
            previous_mode,
        }
    }
    fn update_editor_mode(&mut self, mode: EditorMode) {
        self.previous_mode = self.editor_mode;
        self.editor_mode = mode;
    }
}

pub fn run(file_data: &mut FileData) -> Result<(), Box<dyn Error>> {
    let mut editor_state = EditorState::new(EditorMode::Normal, EditorMode::Normal);

    let mut terminal_tools = termux::TerminalTools::new();

    termux::enable_alternate_buffer()?;
    termux::enable_raw_mode()?;
    termux::clear_screen()?;

    terminal_tools.cursor.move_home()?;

    tui::update_tui(&mut editor_state, &mut terminal_tools)?;
    io::stdout().flush().unwrap();

    let mut command = String::new();
    loop {
        let mut input = [0u8; 1];
        // opening reader gets rid of the shell prompt guy
        io::stdin().read_exact(&mut input)?;

        match editor_state.editor_mode {
            EditorMode::Normal => {
                normal_mode_handler(&input, &mut editor_state, &mut terminal_tools, file_data)?
            }
            EditorMode::Insert => {
                insert_mode_handler(&input, file_data, &mut terminal_tools, &mut editor_state)?
            }
            EditorMode::Visual => {
                normal_mode_handler(&input, &mut editor_state, &mut terminal_tools, file_data)?
            }
            EditorMode::Command => command_mode_handler(
                &input,
                &mut editor_state,
                file_data,
                &mut terminal_tools,
                &mut command,
            )?,
        };
        tui::update_tui(&mut editor_state, &mut terminal_tools)?;
        io::stdout().flush().unwrap();
    }
}

fn normal_mode_handler(
    input: &[u8],
    editor_state: &mut EditorState,
    terminal_tools: &mut termux::TerminalTools,
    file_data: &mut FileData,
) -> Result<(), Box<dyn Error>> {
    match input[0] {
        b':' => {
            editor_state.update_editor_mode(EditorMode::Command);
        }

        b'j' => {
            terminal_tools.cursor.move_down(1)?;
            ()
        }
        b'k' => {
            terminal_tools.cursor.move_up(1)?;
            ()
        }
        b'l' => {
            terminal_tools.cursor.move_right(1)?;
            file_data.file_contents_buffer.move_cursor_right()
        }
        b'h' => {
            terminal_tools.cursor.move_left(1, false)?;
            file_data.file_contents_buffer.move_cursor_left()
        }

        b'i' => {
            editor_state.update_editor_mode(EditorMode::Insert);
        }
        b'v' => editor_state.update_editor_mode(EditorMode::Visual),
        // return/enter
        13 => terminal_tools.cursor.move_down(1)?,
        //backspace
        127 => terminal_tools.cursor.move_left(1, false)?,
        27 => (),
        _ => (),
    };
    Ok(())
}
/// this function handles the parsing of commands recieved from command mode upon recieving input
/// of the Enter key.
fn command_parser(command: &Option<char>, file_data: &mut FileData) -> Result<(), Box<dyn Error>> {
    match command {
        Some(c) => match c {
            'q' => {
                termux::disable_alternate_buffer()?;
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
    input: &[u8],
    editor_state: &mut EditorState,
    file_data: &mut FileData,
    terminal_tools: &mut termux::TerminalTools,
    command: &mut String,
) -> Result<(), Box<dyn Error>> {
    editor_state.previous_mode = EditorMode::Command;

    match input[0] {
        // return/enter key code
        13 => {
            let len = command.len();
            let mut command_chars = command.chars();
            for _ in 0..len {
                match command_parser(&command_chars.next(), file_data) {
                    Ok(_) => (),
                    Err(_) => {
                        write!(io::stdout(), "command not found")?;
                        break;
                    }
                }
            }
            editor_state.update_editor_mode(EditorMode::Normal);
            String::clear(command);
            return Ok(());
        }
        // backspace key code
        127 => {
            command.pop();
            terminal_tools.cursor.backspace(true)?;
            ()
        }
        // code for <C-c> | ascii code for Esc
        3 | 27 => {
            editor_state.update_editor_mode(EditorMode::Normal);
            terminal_tools.cursor.restore_cursor_position()?;
            io::stdout().flush().unwrap();
        }
        _ => {
            command.push(input[0] as char);
            terminal_tools.cursor.write_char(&input[0], true)?;
        }
    };
    Ok(())
}

fn insert_mode_handler(
    input: &[u8],
    file_data: &mut FileData,
    terminal_tools: &mut termux::TerminalTools,
    editor_state: &mut EditorState,
) -> Result<(), Box<dyn Error>> {
    match input[0] {
        // return/enter
        13 => {
            file_data.file_contents_buffer.insert_left('\n');
            terminal_tools.cursor.return_newline()?;
        }
        //backspace
        127 => {
            file_data.file_contents_buffer.delete_char();
            terminal_tools.cursor.backspace(false)?;
        }
        // <C-c> | Esc
        3 | 27 => {
            editor_state.update_editor_mode(EditorMode::Normal);
        }
        _ => {
            file_data.file_contents_buffer.insert_left(input[0] as char);

            terminal_tools.cursor.write_char(&input[0], false)?;
        }
    };
    Ok(())
}

fn save_file_contents(file_name: String, file_content: String) {
    let data = file_content;

    fs::write(format!("./{file_name}"), data).expect("should write to /file_name");
}
