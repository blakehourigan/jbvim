const GROW_SIZE: usize = 200;

#[derive(Debug)]
pub struct GapBuffer {
    buffer: Vec<char>,
    gap_begin: usize,
    gap_end: usize,
}

impl GapBuffer {
    pub fn new(content: Option<String>) -> GapBuffer {
        match content {
            Some(string) => {
                let contents = string;

                let mut gap_buffer = GapBuffer {
                    buffer: vec!['\0'; 200],
                    gap_begin: 0,
                    gap_end: 199,
                };
                // buffer starts at begin of string, so we add exting content to
                // the end by default
                gap_buffer.buffer.extend(contents.chars());
                gap_buffer
            }
            None => GapBuffer {
                buffer: vec!['\0'; 200],
                gap_begin: 0,
                gap_end: 199,
            },
        }
    }
    pub fn insert_left(&mut self, char: char) {
        if self.gap_begin < self.gap_end {
            self.buffer[self.gap_begin] = char;
            self.gap_begin += 1;
        } else {
            self.grow_buffer();
        }
    }
    pub fn move_cursor_right(&mut self) {
        let tmp = self.buffer[self.gap_begin];

        self.buffer[self.gap_begin] = self.buffer[self.gap_end + 1];

        self.buffer[self.gap_end] = tmp;

        self.gap_begin += 1;
        self.gap_end += 1;
    }

    pub fn move_cursor_left(&mut self) {
        if self.gap_begin != 0 {
            let tmp = self.buffer[self.gap_end];

            self.buffer[self.gap_end] = self.buffer[self.gap_begin - 1];

            self.buffer[self.gap_begin - 1] = tmp;

            self.gap_begin -= 1;
            self.gap_end -= 1;
        }
    }

    pub fn delete_char(&mut self) {
        self.gap_begin -= 1;
    }

    // this is completely wrong as of now
    //fn remaining_capacity(&self) -> usize {
    //    self.buffer.capacity() - self.gap_end
    //}

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

    pub fn is_line_end(&self) -> bool {
        match self.buffer.get(self.gap_end + 1) {
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
    /// takes a reference to the gap buffer and returns
    /// a string containing the contents of the current line until either
    /// the newline character '\n' or eof is reached, whichever comes first.
    pub fn grab_to_line_end(&self) -> String {
        let v = &self.buffer[self.gap_end + 1..];
        v.iter().take_while(|c| **c != '\n').collect()
    }
    pub fn find_valid_move(
        &mut self,
        movement: &str,
        curr_line: usize,
        curr_col: usize,
    ) -> Option<(usize, usize)> {
        match movement {
            "down" => {
                //the amount of right moves we take in buffer is length
                let s: String = self.buffer.drain(self.gap_end + 1..).collect();
                let mut iter = s.chars();

                let _ = iter
                    .by_ref()
                    .take_while(|c| *c != '\n')
                    .for_each(|_x| self.move_cursor_right());
                match iter.next() {
                    None => return None,
                    _ => (),
                }

                let mut new_col = 0;
                for i in 1..=curr_col {
                    new_col = i;
                    let value = iter.by_ref().next();
                    match value {
                        Some(_) => (),
                        None => return None,
                    }
                }

                return Some((curr_line + 1, new_col));
            }
            "up" => None,
            "left" => match self.buffer.get((self.gap_begin as i32 - 1) as usize) {
                Some(_) => {
                    self.move_cursor_left();
                    return Some((curr_line, curr_col - 1));
                }
                None => return None,
            },
            "right" => match self.buffer.get(self.gap_end + 1) {
                Some(c) => {
                    if *c == '\n' {
                        return None;
                    } else if *c == '\r' {
                        return None;
                    } else {
                        self.move_cursor_right();
                        return Some((curr_line, curr_col + 1));
                    }
                }
                None => return None,
            },

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
        dbg!(&buffer.buffer);

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
    #[test]
    fn is_valid_test() {
        let mut buffer = GapBuffer::new(Option::None);
        let string1 = "this is\n      ss";

        for c in string1.chars() {
            buffer.insert_left(c);
        }
        for _ in 0..11 {
            buffer.move_cursor_left();
        }

        let result = buffer.find_valid_move("down", 1);
        assert_eq!(result, Some(1));
    }
}
