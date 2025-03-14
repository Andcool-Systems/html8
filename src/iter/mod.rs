use std::cmp;

#[derive(Clone, Debug)]
pub struct Iter<T: Clone> {
    vec: Vec<T>,
    pos: usize,
}

impl<T: Clone> Iter<T> {
    pub fn from<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            vec: iter.into_iter().collect(),
            pos: 0,
        }
    }

    pub fn next(&mut self) -> Option<T> {
        if self.pos >= self.vec.len() {
            return None;
        }

        self.pos += 1;
        Some(self.vec[self.pos - 1].clone())
    }

    pub fn peek(&self) -> Option<T> {
        if self.pos >= self.vec.len() {
            return None;
        }

        Some(self.vec[self.pos].clone())
    }

    pub fn step_back(&mut self) {
        self.pos = cmp::max(0, self.pos - 1)
    }
}
