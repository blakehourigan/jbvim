const GROW_SIZE: usize = 50;
const INITIAL_SIZE: usize = 150;
const CURRENT_ITEM_OFFSET: usize = 1;

#[derive(Debug, Clone)]
pub struct GapBuffer<T> {
    pub buffer: Vec<Option<T>>,
    pub gap_begin: usize,
    pub gap_end: usize,
    pub filled_items: usize,
}

impl<T> GapBuffer<T>
where
    T: Clone + std::fmt::Debug,
{
    pub fn new() -> GapBuffer<T> {
        let gap_buffer = GapBuffer {
            buffer: vec![None; INITIAL_SIZE],
            gap_begin: 0,
            gap_end: INITIAL_SIZE - 1,
            filled_items: 0,
        };
        return gap_buffer;
    }
    fn retrieve_item_mut(&mut self, index: usize) -> &mut Option<T> {
        let buf_len = self.buffer.len();
        let item = self.buffer.get_mut(index);
        match item {
            Some(item) => return item,
            None => {
                print!("error accessing element at {index}, gap begin is at: {}, gap end is at {}, gap length is {}", self.gap_begin, self.gap_end, buf_len);
                panic!("no item there");
            }
        }
    }
    fn retrieve_item(&self, index: usize) -> &Option<T> {
        let buf_len = self.buffer.len();
        let item = self.buffer.get(index);
        match item {
            Some(item) => return item,
            None => {
                print!("error accessing element at {index}, gap begin is at: {}, gap end is at {}, gap length is {}", self.gap_begin, self.gap_end, buf_len);
                panic!("no item there");
            }
        }
    }
    pub fn reset(&mut self) {
        while self.gap_begin != 0 {
            self.move_gap_left();
        }
    }
    pub fn insert_left(&mut self, item: T) {
        if self.gap_begin + 1 == self.gap_end {
            self.grow_buffer();
            *self.retrieve_item_mut(self.gap_begin) = Some(item);
            self.gap_begin += 1;
        } else {
            *self.retrieve_item_mut(self.gap_begin) = Some(item);
            self.gap_begin += 1;
        }
        self.filled_items += 1;
    }
    pub fn delete_item(&mut self) {
        if self.gap_begin != 0 {
            self.retrieve_item_mut(self.gap_begin - 1).take();
            self.gap_begin -= 1;
        }
    }
    /// This function inserts the current cursored item at the beginning of the gap and then moves the gap to the right
    /// If the gaps end is equal the the total length of the buffer, then you cannot move the
    /// buffer any further
    /// What this effectively does is take your current item and make it your previous item,
    /// allowing you to insert after that item
    pub fn move_gap_right(&mut self) {
        if self.gap_end + 1 == self.buffer.len() {
            return;
        }
        let right_item = self.retrieve_item_mut(self.gap_end + 1).take();

        *self.retrieve_item_mut(self.gap_begin) = right_item;

        self.gap_begin += 1;
        self.gap_end += 1;
    }
    /// this function takes the item prior to the gap and inserts it at the end of the gap. Then it
    /// moves the gap beginning and end to the left. this effectively shifts your gap left, making
    /// your previous item your current item which will allow insertion before the current item
    pub fn move_gap_left(&mut self) {
        let left_val = self.retrieve_item_mut(self.gap_begin - 1).take();
        *self.retrieve_item_mut(self.gap_end) = left_val;

        self.gap_begin -= 1;
        self.gap_end -= 1;
    }

    fn grow_buffer(&mut self) {
        let v: Vec<Option<T>> = vec![None; GROW_SIZE];
        self.buffer.splice(self.gap_begin..self.gap_end, v);
        self.gap_end += GROW_SIZE - 1;
    }
}

fn reformat_string(s: &str, max_line_len: usize) -> String {
    let max_line_len = max_line_len - 1;

    let mut c: Vec<char> = s.chars().collect();
    let len = c.len();

    let len = c.len();

    let mut replace = Vec::new();
    let mut j = 0;
    for i in 0..len {
        if c[i] == '\n' {
            j = 0;
        }
        if j >= max_line_len {
            if c[i].is_ascii() && c[i] != ' ' {
                continue;
            } else if c[i] == ' ' {
                replace.push(i);
                j = 0;
            }
        }
        j += 1;
    }

    for i in 0..replace.len() {
        c[replace[i]] = '\n';
    }
    let final_string = c.iter().collect();
    //print!("\n\n\n\n\n\n\n\n\n{:?}", final_string);
    final_string
}

impl GapBuffer<GapBuffer<char>> {
    pub fn build_nested(s: &str, max_line_len: usize) -> GapBuffer<GapBuffer<char>> {
        let s = reformat_string(s, max_line_len);
        let mut content_buffer = GapBuffer::new();

        for line in s.lines() {
            let line_buf = GapBuffer::build(Some(line), false);
            content_buffer.insert_left(line_buf);
        }
        content_buffer.reset();

        content_buffer
    }
    pub fn is_first_line(&self) -> bool {
        if self.gap_begin == 0 {
            true
        } else {
            false
        }
    }
    pub fn is_last_line(&self) -> bool {
        if self.gap_end == self.buffer.len() - 1 {
            true
        } else {
            false
        }
    }
    /// returns the 'line' or item at the given index -1. this -1 is useful because terminal lines
    /// are thought of as 1-indexed. this will offset the desired line to match the gap buffer.
    /// the gap buffer is returned to starting position and moved right line - 1 times to arrive at
    /// desired nested item (line) a reference to this line is returned
    pub fn get_line(&mut self, line: usize) -> &mut GapBuffer<char> {
        let line = line - 1;
        if line > self.filled_items {
            panic!("attempt to access line not in buffer");
        }
        self.reset();
        for _ in 0..line {
            self.move_gap_right();
        }
        self.get_nested()
    }

    pub fn get_content(&self) -> String {
        let mut buffer = self.buffer.clone();
        buffer.retain(|c| c.is_some());
        buffer
            .iter()
            .map(|i| i.as_ref().unwrap().get_content())
            .collect()
    }
    /// this function gets the internal buffer of a nested buffer. the retrieved buffer is found
    /// always at the index of gap_end + 1
    pub fn get_nested(&mut self) -> &mut GapBuffer<char> {
        // get the line after the 'cursor'
        let current_item = self.retrieve_item_mut(self.gap_end + 1);
        match current_item {
            Some(buf) => return buf,
            None => panic!("there is no buffer where you are trying to reach!!!!"),
        }
    }
    pub fn move_line_contents_backspace(&mut self, from: usize, to: usize) {
        let line = self.get_line(from);
        let line_two_content = line.grab_to_end(true);

        // delete removes the PREVIOUS item, so we need to move right to remove the line we are
        // CURRENTLY manipulating
        self.move_gap_right();
        self.delete_item();

        let line = self.get_line(to);

        //line.append_string_to_endline();
        line.move_to_last_char();
        line.insert_left(' ');
        for c in line_two_content.chars() {
            line.insert_left(c);
        }
    }
    pub fn move_line_contents_enter(&mut self, line: usize) {
        ////split off where we pressed enter
        let line_buf = self.get_line(line);

        //get content after the split
        let end_of_line_cntnt = line_buf.grab_to_end(false);

        //move to the end of the line and delete everything we took from the og line
        line_buf.move_to_last_char();

        for _ in 0..end_of_line_cntnt.len() {
            line_buf.delete_item();
        }

        // once we have done that, trim any lefthand whitespace to avoid inserting spaces on the
        // newline
        let end_of_line_cntnt = end_of_line_cntnt.trim_start();

        // create the buffer with this content
        let new_buffer = GapBuffer::build(Some(&end_of_line_cntnt), false);
        // move the buffer containing the linebuffers to the right to insert this AFTER the line
        // that information was pulled from... i.e if pulled from line one, this will make our new
        // current item item #2, then we can insert_left which inserts BEFORE THAT ITEM
        self.move_gap_right();
        self.insert_left(new_buffer);
        // move back left so that once you start inserting again you are inserting to the line you
        // just created
        self.move_gap_left();
        //print!("\n\n\n\n{:?}", self.get_content());
    }
}
impl GapBuffer<char> {
    /// builds a GapBuffer of chars given the contents of a file as a string, optionally set the
    /// 'walk_back' parameter to true to walk the gap backwards s.len() spaces. this is useful for
    /// keeping gap buffer at correct position when moving between lines in editor functions like
    /// 'enter' or 'backspace'
    pub fn build(s: Option<&str>, walk_back: bool) -> GapBuffer<char> {
        let mut buffer = GapBuffer::new();
        let disallowed = ['\0', '\n', '\r'];

        match s {
            Some(s) => {
                let len = s.len();

                for c in s.chars() {
                    if disallowed.contains(&c) {
                        continue;
                    }
                    buffer.insert_left(c);
                }
                buffer.insert_left('\n');

                if walk_back {
                    buffer.walk_back(len);
                } else {
                    buffer.reset();
                }
            }
            None => {
                buffer.insert_left('\n');
                buffer.reset();
            }
        }
        //print!("{:?}", buffer);
        buffer
    }

    fn walk_back(&mut self, len: usize) {
        for _ in 0..len {
            self.walk_back(len);
        }
    }

    pub fn get_content(&self) -> String {
        let mut buffer = self.buffer.clone();
        buffer.retain(|c| c.is_some());
        buffer.iter().map(|i| i.unwrap()).collect()
    }
    pub fn move_to_last_char(&mut self) {
        let len = self.grab_to_end(true).len();

        for _ in 0..len {
            self.move_gap_right();
        }
    }
    /// this function gets the length of the string inside of the buffer structure
    pub fn get_len(&self) -> usize {
        let not_allowed = ['\n', '\0', '\r'];
        let v = self.buffer.clone();
        let len: String = v
            .iter()
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
            .take_while(|c| !not_allowed.contains(c))
            .collect();
        return len.len();
    }
    /// takes a reference to the gap buffer and returns
    /// a string containing the contents of the current line until either
    /// the newline character '\n' or eof is reached, whichever comes first.
    ///
    /// this function is useful over get_content because it does NOT include newline characters,
    /// this allows us to modify the data structure more easily that handling newline chars every
    /// time we do a backspace operation
    pub fn grab_to_end(&mut self, full_line: bool) -> String {
        if full_line {
            self.reset();
        } else {
            ()
        }
        let v = self.buffer.get(self.gap_end + CURRENT_ITEM_OFFSET..);
        match v {
            Some(v) => v
                .iter()
                .filter(|c| c.is_some())
                .map(|c| c.unwrap())
                .take_while(|c| *c != '\n')
                .collect(),
            None => panic!("grabbed something out of bounds here"),
        }
    }
    pub fn is_line_end(&mut self) -> bool {
        if self.get_len() == self.gap_begin {
            true
        } else {
            false
        }
    }

    fn is_last_word(&self) -> bool {
        let start = self.gap_end + 1;
        let end = self.buffer.len();
        let mut result = false;

        let mut word = false;

        for i in start..end {
            match self.buffer.get(i) {
                Some(j) => match j {
                    Some(c) => {
                        if *c == '\n' {
                            result = true;
                            break;
                        }
                        if *c == ' ' {
                            word = true;
                        } else if c.is_ascii() && *c != ' ' && word {
                            result = false;
                            break;
                        } else {
                            continue;
                        }
                    }
                    None => continue,
                },
                None => continue,
            }
        }
        result
    }
    pub fn move_to_next_word(&mut self) -> usize {
        if self.is_last_word() {
            return 0;
        }
        let mut num = 0;

        loop {
            self.move_gap_right();
            let cur_item = self.retrieve_item(self.gap_end + 1).unwrap();
            num += 1;
            if cur_item == ' ' {
                break;
            }
        }
        self.move_gap_right();
        num += 1;
        num
    }
    pub fn is_buf_begin(&self) -> bool {
        if self.gap_begin == 0 {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{self, Write};

    const MAX_LINE_LENGTH: usize = 164;

    fn load_file(file_name: &str) -> GapBuffer<GapBuffer<char>> {
        let content = fs::read_to_string(file_name).unwrap_or_else(|err| {
            print!("error loading file {err}");
            panic!("no file there")
        });

        let buffer = GapBuffer::build_nested(&content, MAX_LINE_LENGTH);
        buffer
    }

    #[test]
    fn load_small_file() {
        let content = fs::read_to_string("small_text.txt").unwrap_or_else(|err| {
            print!("error loading file {err}");
            panic!("no file there")
        });

        let buffer = GapBuffer::build_nested(&content, MAX_LINE_LENGTH);
        fs::write(format!("./{}", "small_result.txt"), buffer.get_content())
            .expect("should write to /file_name");

        let content_actual = buffer.get_content();
        print!("{:?}", content_actual);
        //write!(io::st dout(), "here: {content_actual}").expect("write string to stdout");

        assert_eq!(content, content_actual)
    }

    #[test]
    fn load_large_file() {
        let content = fs::read_to_string("large_text.txt").unwrap_or_else(|err| {
            print!("error loading file {err}");
            panic!("no file there")
        });

        let buffer = GapBuffer::build_nested(&content, MAX_LINE_LENGTH);
        assert_eq!(buffer.get_content(), content)
    }

    #[test]
    fn deletion() {
        let file = "small_text.txt";
        let expected = get_expected(file, 1, 0);
        let expected = String::from(&expected[0..expected.len() - 1]);

        let mut buffer = load_file("small_text.txt");

        let line = buffer.get_line(1);
        line.move_to_last_char();

        line.delete_item();

        let actual = line.grab_to_end(true);

        print!("expected: {}\nactual: {}", expected, actual);

        assert_eq!(expected, actual)
    }

    #[test]
    fn chars_basic_insertion() {
        let mut buffer_of_chars = GapBuffer::new();
        for c in "hello, world!".chars() {
            buffer_of_chars.insert_left(c);
        }

        assert_eq!(buffer_of_chars.get_content(), "hello, world!");
    }

    #[test]
    fn get_line() {
        let file = "small_text.txt";
        let expected = get_expected(file, 1, 0);

        let mut buffer = load_file(file);
        let line = buffer.get_line(1);

        let actual = line.grab_to_end(true);

        assert_eq!(actual, expected);
    }

    fn get_expected(file: &str, line: usize, col: usize) -> String {
        let expected = fs::read_to_string(file).unwrap_or_else(|err| {
            print!("{err}");
            panic!("no file there");
        });
        let mut lines = expected.lines();
        for _ in 0..(line - 1) {
            lines.next().unwrap_or_else(|| {
                print!("no line");
                panic!("no line");
            });
        }
        let expected = lines.next().unwrap_or_else(|| {
            print!("no line");
            panic!("no line");
        });

        let expected = String::from(&expected[col..]);
        expected
    }

    #[test]
    fn grab_from_current() {
        let file = "small_text.txt";
        let expected = get_expected(file, 1, 49);
        let mut buffer = load_file(file);
        let line = buffer.get_line(1);

        for _ in 0..49 {
            line.move_gap_right()
        }

        let actual = line.grab_to_end(false);

        assert_eq!(expected, actual);
    }

    #[test]
    fn backspace_line_to_previous() {
        let file = "large_text.txt";
        let expected_init_line = get_expected(file, 40, 0);
        let expected_additional = get_expected(file, 41, 0);

        let expected = format!("{} {}", expected_init_line, expected_additional);

        let mut buffer = load_file(file);
        buffer.move_line_contents_backspace(41, 40);

        let actual = buffer.get_line(40).grab_to_end(true);

        print!("expected: {}\nactual: {}", expected, actual);

        assert_eq!(expected, actual);
    }

    #[test]
    fn enter_line_to_next() {
        let file = "small_text.txt";
        let line = 1;
        let split = 115;
        let old_line_contents = get_expected(file, line, 0);
        let old_line_contents = String::from(&old_line_contents[0..split]);
        let new_line_contents = get_expected(file, line, split);
        let new_line_contents = new_line_contents.trim_start();

        let expected = format!("{}\n{}", old_line_contents, new_line_contents);

        let mut buffer = load_file(file);
        let line_buf = buffer.get_nested();
        for _ in 0..split {
            line_buf.move_gap_right();
        }
        buffer.move_line_contents_enter(line);

        let actual_ln_1 = buffer.get_line(line).grab_to_end(true);
        let actual_ln_2 = buffer.get_line(line + 1).grab_to_end(true);
        let actual = format!("{}\n{}", actual_ln_1, actual_ln_2);

        assert_eq!(expected, actual);
    }

    #[test]
    fn grab_entire_line() {
        let file = "large_text.txt";
        let line = 6;
        let expected = get_expected(file, line, 0);

        let mut buffer = load_file(file);
        let line = buffer.get_line(line);

        let actual = line.grab_to_end(true);

        assert_eq!(expected, actual);
    }
    #[test]
    fn get_start_next_word() {
        let line = "Lorem Ipsum is simply dummy text of the 
printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, 
when an unknown printer took a galley of type and scrambled it to";
        let mut line_buffer = GapBuffer::build(Some(line), false);

        let expected = line.len() - 2;
        let mut actual = 0;

        for _ in 0..line.len() {
            actual = line_buffer.move_to_next_word();
        }

        assert_eq!(expected, actual)
    }

    #[test]
    fn insert_move_left() {
        let mut buffer_of_buffers = GapBuffer::new();

        let mut buffer_of_chars = GapBuffer::new();

        for c in "hello, world!".chars() {
            buffer_of_chars.insert_left(c);
        }
        for _ in 0.."world!".len() {
            buffer_of_chars.move_gap_left();
        }

        for c in "bruh".chars() {
            buffer_of_chars.insert_left(c);
        }

        for _ in 0.."world!".len() {
            buffer_of_chars.move_gap_right();
        }
        buffer_of_buffers.insert_left(buffer_of_chars);

        //println!("{:?}", buffer_of_buffers.get_content());
        //assert_eq!(buf_string, string);
    }
    #[test]
    fn grow() {
        let mut buffer_of_chars = GapBuffer::new();

        let bar = std::iter::repeat("c").take(200).collect::<String>();

        for c in bar.chars() {
            buffer_of_chars.insert_left(c);
        }
        println!("{:?}", buffer_of_chars.get_content());
    }
    #[test]
    fn move_gap_past_len() {
        let content = fs::read_to_string("small_text.txt").unwrap_or_else(|err| {
            print!("error loading file {err}");
            panic!("no file there")
        });

        let mut buffer = GapBuffer::build_nested(&content, 164);
        fs::write(format!("./{}", "small_result.txt"), buffer.get_content())
            .expect("should write to /file_name");

        loop {
            if buffer.is_last_line() {
                break;
            }
            buffer.move_gap_right();
        }
    }
}
