const GROW_SIZE: usize = 50;
const INITIAL_SIZE: usize = 150;

#[derive(Debug, Clone)]
pub struct GapBuffer<T> {
    pub buffer: Vec<Option<T>>,
    pub gap_begin: usize,
    pub gap_end: usize,
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
        };
        return gap_buffer;
    }
    pub fn reset(&mut self) {
        while self.gap_begin != 0 {
            self.move_gap_left();
        }
    }
    pub fn pad_left(&mut self) {
        if self.gap_begin < self.gap_end {
            self.buffer[self.gap_begin] = None;
            self.gap_begin += 1;
        }
    }
    pub fn insert_left(&mut self, item: T) {
        if self.gap_begin < self.gap_end {
            self.buffer[self.gap_begin] = Some(item);
            self.gap_begin += 1;
        } else {
            self.grow_buffer();
        }
    }
    pub fn move_gap_right(&mut self) {
        if self.gap_end == self.buffer.len() {
            return;
        }
        let right_item = self.buffer[self.gap_end + 1].take();

        self.buffer[self.gap_begin] = right_item;

        self.gap_begin += 1;
        self.gap_end += 1;
    }

    pub fn move_gap_left(&mut self) {
        let left_val = self.buffer[self.gap_begin - 1].take();
        self.buffer[self.gap_end] = left_val;

        self.gap_begin -= 1;
        self.gap_end -= 1;
    }

    pub fn delete_item(&mut self) {
        self.gap_begin -= 1;
    }

    fn grow_buffer(&mut self) {
        for _ in 0..GROW_SIZE {
            self.buffer.splice(self.gap_end..self.gap_end, None);
        }
        self.gap_end += GROW_SIZE - 1;
    }
}
impl GapBuffer<char> {
    pub fn get_content(&mut self) -> String {
        self.buffer.retain(|c| c.is_some());
        self.buffer.iter().map(|i| i.unwrap()).collect()
    }

    pub fn is_line_end(&self) -> bool {
        match self.buffer[self.gap_end + 2] {
            Some(c) => {
                if c == '\n' {
                    return true;
                } else {
                    return false;
                }
            }
            None => return true,
        }
    }
    pub fn is_line_begin(&self) -> bool {
        if self.gap_begin == 0 {
            return true;
        }
        false
    }
    pub fn next_is_empty(&self) -> bool {
        match self.buffer[self.gap_end + 1] {
            Some(c) => {
                if c == '\n' {
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
    pub fn grab_to_end(&self) -> String {
        let v = &self.buffer[(self.gap_end + 1)..];
        v.iter()
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
            .take_while(|c| *c != '\n')
            .collect()
    }
}
impl GapBuffer<GapBuffer<char>> {
    //Gapbuffer{buffer, start, end} -> buffer which is a vector of option<t>
    //
    // in our case, that top layer vuffer is a vector such that Vec<option<gapbuffer<option<char>>>
    //
    // so we need to index into the vec, match on the buf, then match on the char
    //
    pub fn get_content(&mut self) -> String {
        let mut s = String::new();
        for i in 0..self.buffer.len() {
            match &mut self.buffer[i] {
                Some(buf) => {
                    let mut buf = buf.clone();
                    s.push_str(&buf.get_content());
                }
                None => (),
            }
        }
        s
    }
    pub fn get_nested(&mut self) -> &mut GapBuffer<char> {
        //subtract 1 for offset
        match &mut self.buffer[self.gap_end + 1] {
            Some(buf) => return buf,
            None => panic!("no buffer there!!!!"),
        }
    }
    pub fn is_last_item(&self) -> bool {
        if self.gap_begin == self.buffer.len() {
            return true;
        }
        false
    }
    pub fn next_is_empty(&self) -> bool {
        if (self.gap_end + 2) == self.buffer.len() {
            return true;
        }
        let line_buf = match &self.buffer[self.gap_end + 2] {
            Some(buf) => buf,
            None => return true,
        };
        match line_buf.buffer[line_buf.gap_end + 1] {
            Some(c) => {
                if c == '\0' {
                    return true;
                }
                return false;
            }
            None => return true,
        };
    }
    pub fn last_is_first(&self) -> bool {
        // if true THIS is first
        if (self.gap_begin) == 0 {
            return true;
        } else {
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        println!("{:?}", buffer_of_buffers.get_content());
        //assert_eq!(buf_string, string);
    }
}
