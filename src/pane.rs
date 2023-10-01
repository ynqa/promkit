use crate::grapheme::Graphemes;

pub struct Pane {
    pub layout: Vec<Graphemes>,
    pub offset: usize,
}

impl Pane {
    pub fn extract(&self, viewport_height: usize) -> Vec<Graphemes> {
        if self.layout.len() <= viewport_height {
            return self.layout.clone();
        }
        let mut start = self.offset;
        let end = self.offset + viewport_height;
        if end > self.layout.len() {
            start = self.layout.len().saturating_sub(viewport_height);
        }

        return self
            .layout
            .iter()
            .enumerate()
            .filter(|(i, _)| start <= *i && *i < end)
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
    }
}

// #[cfg(test)]
// mod test {
//     mod extract {

//         use super::super::*;

//         #[test]
//         fn test() {
//             let expect = vec![Graphemes::from("aa")];
//             assert_eq!(
//                 expect,
//                 Pane::new(
//                     2,
//                     &TextBuffer {
//                         buf: Graphemes::from("aaa "),
//                         position: 0,
//                     },
//                     &Graphemes::from(""),
//                 )
//                 .extract(1)
//             );
//         }

//         #[test]
//         fn test_extract_front() {
//             let expect = vec![Graphemes::from("aa"); 5];
//             assert_eq!(
//                 expect,
//                 Pane::new(
//                     2,
//                     &TextBuffer {
//                         buf: Graphemes::from("a".repeat(100)),
//                         position: 100,
//                     },
//                     &Graphemes::from(""),
//                 )
//                 .extract(5)
//             );
//         }

//         #[test]
//         fn test_extract_buck() {
//             let expect = vec![Graphemes::from("ab"), Graphemes::from("c")];
//             assert_eq!(
//                 expect,
//                 Pane::new(
//                     2,
//                     &TextBuffer {
//                         buf: Graphemes::from("abc"),
//                         position: 0,
//                     },
//                     &Graphemes::from(""),
//                 )
//                 .extract(5)
//             );
//         }
//     }

//     mod matrixify {
//         use super::super::*;

//         #[test]
//         fn test() {
//             let expect = vec![
//                 Graphemes::from(">>"),
//                 Graphemes::from(" a"),
//                 Graphemes::from("aa"),
//                 Graphemes::from(" "),
//             ];
//             assert_eq!(
//                 expect,
//                 Pane::new(
//                     2,
//                     &TextBuffer {
//                         buf: Graphemes::from("aaa "),
//                         position: 0,
//                     },
//                     &Graphemes::from(">> "),
//                 )
//                 .layout
//             );
//         }

//         #[test]
//         fn test_with_emoji() {
//             let expect = vec![
//                 Graphemes::from(">>"),
//                 Graphemes::from(" "),
//                 Graphemes::from("😎"),
//                 Graphemes::from("😎"),
//                 Graphemes::from(" "),
//             ];
//             assert_eq!(
//                 expect,
//                 Pane::new(
//                     2,
//                     &TextBuffer {
//                         buf: Graphemes::from("😎😎 "),
//                         position: 0,
//                     },
//                     &Graphemes::from(">> "),
//                 )
//                 .layout
//             );
//         }

//         #[test]
//         fn test_with_emoji_at_narrow_terminal() {
//             let expect = vec![
//                 Graphemes::from(">"),
//                 Graphemes::from(">"),
//                 Graphemes::from(" "),
//                 Graphemes::from(" "),
//             ];
//             assert_eq!(
//                 expect,
//                 Pane::new(
//                     1,
//                     &TextBuffer {
//                         buf: Graphemes::from("😎😎 "),
//                         position: 0,
//                     },
//                     &Graphemes::from(">> "),
//                 )
//                 .layout
//             );
//         }
//     }
// }
