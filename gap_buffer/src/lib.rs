use core::fmt;

const GROW_SIZE: usize = 50;
// cursor highlights character 1 AFTER the gaps end
const CURSOR_OFFSET: usize = 1;

pub struct GapBuffer {
    buffer: Vec<char>,
    gap_begin: usize,
    gap_end: usize,
}

impl fmt::Debug for GapBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = String::new();
        for c in &self.buffer {
            buffer.push(*c);
        }
        f.debug_struct("GapBuffer")
            .field("buf", &buffer)
            .field("begin", &self.gap_begin)
            .field("end", &self.gap_end)
            .finish()
    }
}
impl GapBuffer {
    pub fn new(content: Option<String>) -> GapBuffer {
        let mut gap_buffer = GapBuffer {
            buffer: vec!['\0'; 75],
            gap_begin: 0,
            gap_end: 74,
        };

        if let Some(string) = content {
            let contents = string;
            gap_buffer.buffer.extend(contents.chars());
        };

        gap_buffer
    }
    pub fn insert_left(&mut self, char: char) {
        if self.gap_begin < self.gap_end {
            self.buffer[self.gap_begin] = char;
            self.gap_begin += 1;
        } else {
            self.grow_buffer();
        }
    }
    pub fn move_gap_right(&mut self) {
        match self.buffer.get(self.gap_end + 1) {
            Some(c) => {
                let tmp = self.buffer[self.gap_begin];

                self.buffer[self.gap_begin] = *c;

                self.buffer[self.gap_end + 1] = tmp;

                self.gap_begin += 1;
                self.gap_end += 1;
            }
            None => panic!("this should be caught by fn is_line_end"),
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.gap_begin < 1 {
            return;
        }
        let tmp = self.buffer[self.gap_end];

        self.buffer[self.gap_end] = match self.buffer.get(self.gap_begin - 1) {
            Some(c) => *c,
            None => panic!("should be a char there, that's a problem!"),
        };

        self.buffer[self.gap_begin - 1] = tmp;

        self.gap_begin -= 1;
        self.gap_end -= 1;
    }

    pub fn delete_char(&mut self) {
        self.gap_begin -= 1;
    }

    fn grow_buffer(&mut self) {
        let new_items = std::iter::repeat('\0')
            .take(GROW_SIZE)
            .collect::<Vec<char>>();
        self.buffer.splice(self.gap_end..self.gap_end, new_items);
        self.gap_end += GROW_SIZE;
    }

    pub fn get_content(&mut self) -> String {
        drop(self.buffer.drain(self.gap_begin..=self.gap_end));

        self.buffer.iter().collect()
    }

    pub fn is_line_beginning(&self) -> bool {
        if self.gap_begin < 1 {
            return true;
        };
        match self.buffer.get(self.gap_begin - 1) {
            Some(c) => {
                if *c == '\n' {
                    return true;
                } else {
                    return false;
                }
            }
            None => return true,
        }
    }
    pub fn next_is_eof(&self) -> bool {
        match self.buffer.get(self.gap_end + CURSOR_OFFSET + 1) {
            Some(_) => {
                return false;
            }
            None => return true,
        }
    }
    pub fn is_first_line(&self) -> bool {
        let mut i = 0;
        if self.gap_begin == 0 {
            return true;
        }
        loop {
            match self.buffer.get(self.gap_begin - i) {
                Some(c) => {
                    if *c == '\n' {
                        return false;
                    }
                }
                None => return true,
            }
            i += 1;
        }
    }
    pub fn is_last_line(&self) -> bool {
        let mut i = 0;
        loop {
            match self.buffer.get(self.gap_end + CURSOR_OFFSET + i) {
                Some(c) => {
                    if *c == '\n' {
                        match self.buffer.get(self.gap_end + CURSOR_OFFSET + i + 1) {
                            None => return true,
                            _ => break,
                        }
                    } else {
                        i += 1;
                    }
                }
                None => return true,
            }
        }
        return false;
    }
    pub fn next_is_line_end(&self) -> bool {
        match self.buffer.get(self.gap_end + CURSOR_OFFSET + 1) {
            Some(c) => {
                if *c == '\n' {
                    return true;
                } else {
                    return false;
                }
            }
            None => return true,
        }
    }
    pub fn cursor_is_newline(&self) -> bool {
        match self.buffer.get(self.gap_end + CURSOR_OFFSET) {
            Some(c) => {
                if *c == '\n' {
                    return true;
                } else {
                    return false;
                }
            }
            None => return true,
        }
    }
    pub fn before_cursor_is_newline(&self) -> bool {
        if self.gap_begin < 1 {
            return true;
        }
        match self.buffer.get(self.gap_begin - 1) {
            Some(c) => {
                if *c == '\n' {
                    return true;
                } else {
                    return false;
                }
            }
            None => return true,
        }
    }
    pub fn next_is_space(&self) -> bool {
        match self.buffer.get(self.gap_end + CURSOR_OFFSET + 1) {
            Some(c) => {
                if *c == ' ' {
                    return true;
                } else {
                    return false;
                }
            }
            None => return true,
        }
    }
    pub fn get_before_after_buffer(&self) {
        if self.gap_begin > 0 {
            print!(
                "before{:?}, after{:?}",
                self.buffer.get(self.gap_begin - 1),
                self.buffer.get(self.gap_end + CURSOR_OFFSET)
            );
        }
    }
    /// takes a reference to the gap buffer and returns
    /// a string containing the contents of the current line until either
    /// the newline character '\n' or eof is reached, whichever comes first.

    pub fn grab_to_line_end(&self) -> String {
        let v = &self.buffer[(self.gap_end + CURSOR_OFFSET)..];
        v.iter().take_while(|c| **c != '\n').collect()
    }
    /// takes a reference to self, movement selection, current line, current col, and decides
    /// whether the desires move is valid based on the current structure of the GapBuffer.
    /// Returns a Option<(usize, usize)>. This value is None if the move is invalid, or the tuple
    /// is returned in the form of (new_line, new_column).

    pub fn find_valid_move(
        &mut self,
        movement: &str,
        curr_line: usize,
        curr_col: usize,
    ) -> Option<(usize, usize)> {
        match movement {
            "up" => {
                if curr_line == 1 {
                    return None;
                }
                let cursor_begin = self.gap_begin;
                let mut v = self.buffer[..cursor_begin].iter().rev();

                let end_line = v.by_ref().position(|i| *i == '\n');
                let start_line: Vec<_> = v.by_ref().take_while(|i| **i != '\n').collect();
                let start_line = start_line.len();

                let prev_line = &self.buffer
                    [cursor_begin - (start_line + end_line?)..=(cursor_begin - end_line?) - 1];

                let mut new_col = curr_col;

                for i in (0..=curr_col).rev() {
                    if i == 0 {
                        return None;
                    }

                    // curr col is 1 based indexed, need to subtract that off for accuracy
                    match prev_line.get(i - 1) {
                        Some(_) => {
                            new_col = i;
                            break;
                        }
                        None => (),
                    }
                }
                // move the cursor past the newline that we found, then to the new column - 1
                // because the terminal cursor is 1 based
                let movement_left = (start_line + end_line? + 1) - (new_col - 1);
                for _ in 0..movement_left {
                    self.move_cursor_left();
                }
                return Some((curr_line - 1, new_col));
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn insert_func() {
        let mut buffer = GapBuffer::new(Option::None);
        let string = "this is";

        for c in string.chars() {
            buffer.insert_left(c);
        }
        assert_eq!(buffer.get_content(), String::from("this is"));
    }

    #[test]
    fn delete_func() {
        let mut buffer = GapBuffer::new(Option::None);
        let string = "chis is";
        buffer.grow_buffer();

        buffer.insert_left('t');
        buffer.delete_char();

        for c in string.chars() {
            buffer.insert_left(c);
        }

        assert_eq!(buffer.get_content(), String::from("chis is"));
    }

    #[test]
    fn move_cursor_left_func() {
        let mut buffer = GapBuffer::new(Option::None);
        let string1 = "chis is";
        let string2 = "em";

        buffer.grow_buffer();

        buffer.insert_left('t');
        buffer.delete_char();

        for c in string1.chars() {
            buffer.insert_left(c);
        }

        for _ in 0..3 {
            buffer.move_cursor_left();
        }
        buffer.delete_char();
        buffer.delete_char();

        for c in string2.chars() {
            buffer.insert_left(c);
        }
        dbg!(&buffer.buffer);
        assert_eq!(buffer.get_content(), String::from("chem is"));
    }

    #[test]
    fn move_cursor_right_func() {
        let mut buffer = GapBuffer::new(Option::None);
        let string1 = "chis is";
        let string2 = "em";

        buffer.insert_left('t');
        buffer.delete_char();

        for c in string1.chars() {
            buffer.insert_left(c);
        }

        for _ in 0..3 {
            buffer.move_cursor_left();
        }

        buffer.delete_char();
        buffer.delete_char();

        for c in string2.chars() {
            buffer.insert_left(c);
        }
        dbg!(&buffer.buffer);

        for _ in 0..3 {
            buffer.move_cursor_right();
        }
        buffer.insert_left('.');

        assert_eq!(buffer.get_content(), String::from("chem is."));
    }

    #[test]
    fn file_loading() {
        let mut file = File::open("text.txt").expect("should open text.txt");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("should read text.txt");

        let mut buffer = GapBuffer::new(Some(contents.clone()));

        assert_eq!(buffer.get_content(), contents);
    }
    #[test]
    fn file_editing() {
        let mut file = File::open("text.txt").expect("should open text.txt");
        let mut contents = String::new();
        let string1 = "hello!\n";

        file.read_to_string(&mut contents)
            .expect("should read text.txt");

        let mut buffer = GapBuffer::new(Some(contents.clone()));
        for c in string1.chars() {
            buffer.insert_left(c);
        }

        let mut cmp_file =
            File::open("modified_text.txt").expect("should open file to compare against");

        let mut cmp_content = String::new();
        cmp_file
            .read_to_string(&mut cmp_content)
            .expect("should read cmp file to string");

        assert_eq!(buffer.get_content(), cmp_content);
    }

    #[test]
    fn inserting_past_buffer() {
        let mut file = File::open("text.txt").expect("should open text.txt");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("should read text.txt");

        let mut buffer = GapBuffer::new(Some(contents));
        for _ in 0..205 {
            buffer.insert_left('h');
        }
        for _ in 0..54 {
            buffer.insert_left('b');
        }

        let mut cmp_file =
            File::open("modified_text2.txt").expect("should open file to compare against");
        let mut cmp_content = String::new();
        cmp_file
            .read_to_string(&mut cmp_content)
            .expect("should read cmp file to string");

        assert_eq!(buffer.get_content(), cmp_content);
    }
}
