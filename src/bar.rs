#[derive(PartialEq, Eq)]
pub(crate) struct Input {
    text: String,
    cursor: usize,
}

impl Input {
    pub(crate) fn text(&self) -> String {
        self.text.clone()
    }

    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }

    pub(crate) fn empty() -> Self {
        Input {
            text: "".to_owned(),
            cursor: 0,
        }
    }

    pub(crate) fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += 1;
        }
    }

    pub(crate) fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub(crate) fn write(&mut self, input: char) {
        self.text.insert(self.cursor, input);
        self.cursor += 1;
    }

    pub(crate) fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.text.remove(self.cursor);
        }
    }

    pub(crate) fn delete(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }
}
