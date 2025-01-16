const GROW_SIZE: usize = 50;

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

    fn remaining_capacity(&self) -> usize {
        self.buffer.capacity() - self.gap_end
    }

    fn grow_buffer(&mut self) {
        //self.buffer.resize(self.buffer.capacity() + GROW_SIZE, '\0');
        let new_items = std::iter::repeat('\0')
            .take(GROW_SIZE)
            .collect::<Vec<char>>();
        self.buffer
            .splice(self.gap_end..self.gap_end, new_items.iter().cloned());
        self.gap_end += GROW_SIZE;
    }
    pub fn get_content(&mut self) -> String {
        drop(self.buffer.drain(self.gap_begin..=self.gap_end));

        self.buffer.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_func() {
        let mut buffer = GapBuffer::new(Option::None);
        buffer.insert_left('t');
        buffer.insert_left('h');
        buffer.insert_left('i');
        buffer.insert_left('s');
        buffer.insert_left(' ');
        buffer.insert_left('i');
        buffer.insert_left('s');
        assert_eq!(buffer.get_content(), String::from("this is"));
    }

    #[test]
    fn delete_func() {
        let mut buffer = GapBuffer::new(Option::None);
        buffer.grow_buffer();
        buffer.insert_left('t');
        buffer.delete_char();
        buffer.insert_left('c');
        buffer.insert_left('h');
        buffer.insert_left('i');
        buffer.insert_left('s');
        buffer.insert_left(' ');
        buffer.insert_left('i');
        buffer.insert_left('s');
        assert_eq!(buffer.get_content(), String::from("chis is"));
    }

    #[test]
    fn move_cursor_left_func() {
        let mut buffer = GapBuffer::new(Option::None);
        buffer.grow_buffer();
        buffer.insert_left('t');
        buffer.delete_char();
        buffer.insert_left('c');
        buffer.insert_left('h');
        buffer.insert_left('i');
        buffer.insert_left('s');
        buffer.insert_left(' ');
        buffer.insert_left('i');
        buffer.insert_left('s');
        dbg!(&buffer.buffer);
        buffer.move_cursor_left();
        buffer.move_cursor_left();
        buffer.move_cursor_left();
        buffer.delete_char();
        buffer.delete_char();
        buffer.insert_left('e');
        buffer.insert_left('m');
        dbg!(&buffer.buffer);
        assert_eq!(buffer.get_content(), String::from("chem is"));
    }

    #[test]
    fn move_cursor_right_func() {
        let mut buffer = GapBuffer::new(Option::None);
        buffer.insert_left('t');
        buffer.delete_char();
        buffer.insert_left('c');
        buffer.insert_left('h');
        buffer.insert_left('i');
        buffer.insert_left('s');
        buffer.insert_left(' ');
        buffer.insert_left('i');
        buffer.insert_left('s');
        buffer.move_cursor_left();
        buffer.move_cursor_left();
        buffer.move_cursor_left();
        buffer.delete_char();
        buffer.delete_char();
        buffer.insert_left('e');
        buffer.insert_left('m');
        dbg!(&buffer.buffer);
        buffer.move_cursor_right();
        buffer.move_cursor_right();
        buffer.move_cursor_right();
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
        file.read_to_string(&mut contents)
            .expect("should read text.txt");

        let mut buffer = GapBuffer::new(Some(contents.clone()));
        buffer.insert_left('h');
        buffer.insert_left('e');
        buffer.insert_left('l');
        buffer.insert_left('l');
        buffer.insert_left('o');
        buffer.insert_left('!');
        buffer.insert_left('\n');

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
        for _ in 0..55 {
            buffer.insert_left('b');
        }
        dbg!(&buffer);

        let mut cmp_file =
            File::open("modified_text2.txt").expect("should open file to compare against");
        let mut cmp_content = String::new();
        cmp_file
            .read_to_string(&mut cmp_content)
            .expect("should read cmp file to string");

        assert_eq!(buffer.get_content(), cmp_content);
    }
}
