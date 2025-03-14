#[derive(Clone, Debug)]
pub struct StringIter {
    vec: Vec<char>,
    pos: usize,
}

impl StringIter {
    pub fn from(dest: &String) -> Self {
        Self {
            vec: dest.chars().collect::<Vec<char>>(),
            pos: 0,
        }
    }

    pub fn next(&mut self) -> Option<char> {
        if self.pos >= self.vec.len() {
            return None;
        }

        self.pos += 1;
        Some(self.vec[self.pos - 1])
    }

    pub fn peek(&self) -> Option<char> {
        if self.pos >= self.vec.len() {
            return None;
        }

        Some(self.vec[self.pos].clone())
    }

    pub fn step_back(&mut self) {
        self.pos -= 1;
    }
}
