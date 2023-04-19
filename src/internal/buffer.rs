use std::cell::Cell;

use crate::grapheme::{Grapheme, Graphemes};

/// Store the user inputs.
#[derive(Debug, Clone, Default)]
pub struct Buffer {
    pub data: Graphemes,
    pub position: Cell<usize>,
}

impl Buffer {
    pub fn position(&self) -> usize {
        self.position.get()
    }

    pub fn prev(&self) -> bool {
        if 0 < self.position.get() {
            self.position.set(self.position.get() - 1);
            return true;
        }
        false
    }

    // e.g. "" == 0, "a" == 1
    pub fn next(&self) -> bool {
        if self.position.get() < self.data.len() {
            self.position.set(self.position.get() + 1);
            return true;
        }
        false
    }

    pub fn to_head(&self) {
        self.position.set(0)
    }

    pub fn to_tail(&self) {
        self.position.set(self.data.len())
    }

    pub fn width_in_position(&self) -> usize {
        if self.position() < 1 {
            return 0;
        }
        match self.data.get(self.position() - 1) {
            Some(g) => g.width,
            None => 0,
        }
    }

    pub fn width_to_position(&self) -> usize {
        self.data
            .iter()
            .enumerate()
            .filter(|&(i, _)| i < self.position())
            .fold(0, |mut m, (_, g)| {
                m += g.width;
                m
            })
    }

    pub fn width_from_position(&self) -> usize {
        self.data
            .iter()
            .enumerate()
            .filter(|&(i, _)| i >= self.position())
            .fold(0, |mut m, (_, g)| {
                m += g.width;
                m
            })
    }

    pub fn replace(&mut self, new: &Graphemes) {
        self.data = new.to_owned();
        self.position.set(new.len());
    }

    pub fn insert(&mut self, ch: Grapheme) {
        let position = self.position();
        self.data.insert(position, ch);
        self.next();
    }

    pub fn overwrite(&mut self, ch: Grapheme) {
        if self.position.get() == self.data.len() {
            self.insert(ch);
        } else {
            let position = self.position();
            self.data.splice(position..position + 1, vec![ch]);
            self.next();
        }
    }

    /// Erase a char in the current position.
    pub fn erase(&mut self) {
        if self.position.get() != 0 {
            self.prev();
            let position = self.position();
            self.data.drain(position..position + 1);
        }
    }
}

#[cfg(test)]
mod test {
    use std::cell::Cell;

    use super::{Buffer, Grapheme, Graphemes};

    #[test]
    fn width_in_position() {
        let b = Buffer {
            data: Graphemes(vec![
                Grapheme { ch: 'a', width: 10 },
                Grapheme { ch: 'b', width: 20 },
                Grapheme { ch: 'c', width: 30 },
            ]),
            position: Cell::new(2), // indicate `b`.
        };
        assert_eq!(20, b.width_in_position());
    }

    #[test]
    fn width_to_position() {
        let b = Buffer {
            data: Graphemes(vec![
                Grapheme { ch: 'a', width: 1 },
                Grapheme { ch: 'b', width: 10 },
                Grapheme {
                    ch: 'c',
                    width: 100,
                },
            ]),
            position: Cell::new(2), // indicate `b`.
        };
        assert_eq!(11, b.width_to_position());
    }

    #[test]
    fn width_from_position() {
        let b = Buffer {
            data: Graphemes(vec![
                Grapheme { ch: 'a', width: 1 },
                Grapheme { ch: 'b', width: 10 },
                Grapheme {
                    ch: 'c',
                    width: 100,
                },
            ]),
            position: Cell::new(2), // indicate `b`.
        };
        assert_eq!(100, b.width_from_position());
    }

    #[test]
    fn replace() {
        let mut b = Buffer {
            data: Graphemes::from("abc"),
            ..Default::default()
        };

        b.replace(&Graphemes::from("abcde"));

        assert_eq!(Graphemes::from("abcde"), b.data);
        assert_eq!(5, b.position.get());
    }

    #[test]
    fn insert() {
        let mut b = Buffer {
            data: Graphemes::from("abcde"),
            position: Cell::new(4), // indicate `d`.
        };

        b.insert(Grapheme::from('0'));

        assert_eq!(Graphemes::from("abcd0e"), b.data);
        assert_eq!(5, b.position.get());
    }

    #[test]
    fn overwrite() {
        let mut b = Buffer {
            data: Graphemes::from("abcde"),
            position: Cell::new(3), // indicate `c`.
        };

        b.overwrite(Grapheme::from('0'));

        assert_eq!(Graphemes::from("abc0e"), b.data);
        assert_eq!(4, b.position.get());
    }

    #[test]
    fn overwrite_on_last_char() {
        let mut b = Buffer {
            data: Graphemes::from("abcde"),
            position: Cell::new(5), // indicate `e`.
        };

        b.overwrite(Grapheme::from('0'));

        assert_eq!(Graphemes::from("abcde0"), b.data);
        assert_eq!(6, b.position.get());
    }

    #[test]
    fn erase() {
        let mut b = Buffer {
            data: Graphemes::from("abcde"),
            position: Cell::new(4), // indicate `d`.
        };

        b.erase();

        assert_eq!(Graphemes::from("abce"), b.data);
        assert_eq!(3, b.position.get());
    }

    #[test]
    fn erase_for_empty() {
        let mut b = Buffer::default();
        b.erase();
        assert_eq!(Graphemes::from(""), b.data);
        assert_eq!(0, b.position.get());
    }
}
