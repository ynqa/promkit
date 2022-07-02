use std::ops::{Deref, DerefMut};

use crate::{
    edit::{Cursor, Editor},
    grapheme::{Grapheme, Graphemes},
};

/// Store the user inputs.
#[derive(Debug, Clone, Default)]
pub struct Buffer(pub Editor<Graphemes>);

impl Deref for Buffer {
    type Target = Editor<Graphemes>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Cursor for Buffer {
    fn pos(&self) -> usize {
        self.idx.get()
    }

    fn prev(&self) -> bool {
        if 0 < self.idx.get() {
            self.idx.set(self.idx.get() - 1);
            return true;
        }
        false
    }

    // e.g. "" == 0, "a" == 1
    fn next(&self) -> bool {
        if self.idx.get() < self.data.len() {
            self.idx.set(self.idx.get() + 1);
            return true;
        }
        false
    }

    fn to_head(&self) {
        self.idx.set(0)
    }

    fn to_tail(&self) {
        self.idx.set(self.data.len())
    }
}

impl Buffer {
    pub fn width_in_pos(&self) -> usize {
        if self.pos() < 1 {
            return 0;
        }
        match self.data.get(self.pos() - 1) {
            Some(g) => g.width,
            None => 0,
        }
    }

    pub fn width_to_pos(&self) -> usize {
        self.data
            .iter()
            .enumerate()
            .filter(|&(i, _)| i < self.pos())
            .fold(0, |mut m, (_, g)| {
                m += g.width;
                m
            })
    }

    pub fn width_from_pos(&self) -> usize {
        self.data
            .iter()
            .enumerate()
            .filter(|&(i, _)| i >= self.pos())
            .fold(0, |mut m, (_, g)| {
                m += g.width;
                m
            })
    }

    pub fn replace(&mut self, new: &Graphemes) {
        self.data = new.to_owned();
        self.idx.set(new.len());
    }

    pub fn insert(&mut self, ch: Grapheme) {
        let pos = self.pos();
        self.data.insert(pos, ch);
        self.next();
    }

    pub fn overwrite(&mut self, ch: Grapheme) {
        if self.idx.get() == self.data.len() {
            self.insert(ch);
        } else {
            let pos = self.pos();
            self.data.splice(pos..pos + 1, vec![ch]);
            self.next();
        }
    }

    /// Erase a char in the current position.
    pub fn erase(&mut self) {
        if self.idx.get() != 0 {
            self.prev();
            let pos = self.pos();
            self.data.drain(pos..pos + 1);
        }
    }
}

#[cfg(test)]
mod test {
    use std::cell::Cell;

    use super::{Buffer, Editor, Grapheme, Graphemes};

    #[test]
    fn width_in_pos() {
        let b = Buffer(Editor::<Graphemes> {
            data: Graphemes(vec![
                Grapheme { ch: 'a', width: 10 },
                Grapheme { ch: 'b', width: 20 },
                Grapheme { ch: 'c', width: 30 },
            ]),
            idx: Cell::new(2), // indicate `b`.
        });
        assert_eq!(20, b.width_in_pos());
    }

    #[test]
    fn replace() {
        let mut b = Buffer(Editor::<Graphemes> {
            data: Graphemes::from("abc"),
            ..Default::default()
        });

        b.replace(&Graphemes::from("abcde"));

        assert_eq!(Graphemes::from("abcde"), b.data);
        assert_eq!(5, b.idx.get());
    }

    #[test]
    fn insert() {
        let mut b = Buffer(Editor::<Graphemes> {
            data: Graphemes::from("abcde"),
            idx: Cell::new(4), // indicate `d`.
        });

        b.insert(Grapheme::from('0'));

        assert_eq!(Graphemes::from("abcd0e"), b.data);
        assert_eq!(5, b.idx.get());
    }

    #[test]
    fn overwrite() {
        let mut b = Buffer(Editor::<Graphemes> {
            data: Graphemes::from("abcde"),
            idx: Cell::new(3), // indicate `c`.
        });

        b.overwrite(Grapheme::from('0'));

        assert_eq!(Graphemes::from("abc0e"), b.data);
        assert_eq!(4, b.idx.get());
    }

    #[test]
    fn overwrite_on_last_char() {
        let mut b = Buffer(Editor::<Graphemes> {
            data: Graphemes::from("abcde"),
            idx: Cell::new(5), // indicate `e`.
        });

        b.overwrite(Grapheme::from('0'));

        assert_eq!(Graphemes::from("abcde0"), b.data);
        assert_eq!(6, b.idx.get());
    }

    #[test]
    fn erase() {
        let mut b = Buffer(Editor::<Graphemes> {
            data: Graphemes::from("abcde"),
            idx: Cell::new(4), // indicate `d`.
        });

        b.erase();

        assert_eq!(Graphemes::from("abce"), b.data);
        assert_eq!(3, b.idx.get());
    }

    #[test]
    fn erase_for_empty() {
        let mut b = Buffer::default();
        b.erase();
        assert_eq!(Graphemes::from(""), b.data);
        assert_eq!(0, b.idx.get());
    }
}
