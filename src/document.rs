use crossterm::event::KeyCode;

#[derive(Debug, Default)]
struct Document {
    pub text: String,
    cursor_position: i32,
    last_key: Option<KeyCode>,
}

impl Document {
    fn new() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            ..Default::default()
        }
    }

    pub fn cursor_position(&self) -> i32 {
        self.cursor_position
    }

    pub fn last_key_stroke(&self) -> Option<KeyCode> {
        self.last_key
    }

    /// Returns the cursor position on rendered text on terminal emulators.
    /// So if Document is "日本(cursor)語", DisplayedCursorPosition returns 4 because '日' and '本'
    /// are double width characters.
    fn display_cursor_position(&self) -> i32 {
        let mut position = 0;
        let (t, _) = self.text.split_at(self.cursor_position as usize);
        for i in t.chars() {
            position += i.len_utf8()
        }
        position as i32
    }

    /// Return character relative to cursor position, or empty string
    // TODO: return type should be option, since it is possible for the string to empty
    // TODO: offset should be a unsigned num data type
    fn get_char_relative_to_cursor(&self, offset: i32) -> char {
        let mut s = self.text.clone();
        let mut count = 0;
        while s.len() > 0 {
            count += 1;
            let temp = s.chars().take(1)
                .map(|c| (c, c.len_utf8()))
                .collect::<Vec<_>>();
            let (c, size) = temp.first().unwrap();
            if count == self.cursor_position+offset {
                return c.clone();
            }
            s = s.split_at(*size).1.to_string();
        }
        char::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_char_relative_to_cursor() {
        assert_eq!('e', Document {
            text: "line 1\nline 2\nline 3\nline 4\n".to_string(),
            cursor_position: ("line 1\n".to_string() + "lin").chars().count() as i32,
            ..Default::default()
        }.get_char_relative_to_cursor(1));
        assert_eq!('く', Document {
            text: "あいうえお\nかきくけこ\nさしすせそ\nたちつてと\n".to_string(),
            cursor_position: 8,
            ..Default::default()
        }.get_char_relative_to_cursor(1));
        assert_eq!('н', Document {
            text: "Добрый\nдень\nДобрый день".to_string(),
            cursor_position: 9,
            ..Default::default()
        }.get_char_relative_to_cursor(1));
    }
}