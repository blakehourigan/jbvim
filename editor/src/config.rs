use crate::tui;
use gap_buffer::GapBuffer;
use std::fs;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EditorMode {
    Normal,
    Insert,
    Visual,
    Command,
    ShutDown,
}

impl EditorMode {
    pub fn value(&self) -> String {
        match *self {
            EditorMode::Normal => String::from("normal"),
            EditorMode::Insert => String::from("insert"),
            EditorMode::Visual => String::from("visual"),
            EditorMode::Command => String::from("command"),
            EditorMode::ShutDown => String::from(""),
        }
    }
}

pub struct FileData {
    pub file_name: String,
    pub file_handle: fs::File,
    pub file_contents_buffer: GapBuffer,
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
        tui::write_existing_file(&file_contents);
        //}
        let file_contents_buffer = GapBuffer::new(Some(file_contents));

        Ok(FileData {
            file_name,
            file_handle,
            file_contents_buffer,
        })
    }
    pub fn save_file_contents(&mut self) {
        let data = self.file_contents_buffer.get_content();

        fs::write(format!("./{}", self.file_name), data).expect("should write to /file_name");
    }
}
pub struct EditorState {
    pub editor_mode: EditorMode,
    pub previous_mode: EditorMode,
}

impl EditorState {
    pub fn new(editor_mode: EditorMode, previous_mode: EditorMode) -> Self {
        EditorState {
            editor_mode,
            previous_mode,
        }
    }
    pub fn update_editor_mode(&mut self, mode: EditorMode) {
        self.previous_mode = self.editor_mode;
        self.editor_mode = mode;
    }
    pub fn get_current_mode(&self) -> EditorMode {
        self.editor_mode
    }
}
