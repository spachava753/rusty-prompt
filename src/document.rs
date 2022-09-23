use std::cmp::Ordering;
use std::ops::Index;
use crossterm::event::KeyCode;
use unicode_width::UnicodeWidthChar;

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
    fn display_cursor_position(&self) -> usize {
        self.text.chars()
            .take(self.cursor_position as usize)
            .map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
            .sum()
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
            if count == self.cursor_position + offset {
                return c.clone();
            }
            s = s.split_at(*size).1.to_string();
        }
        char::default()
    }

    /// Returns the text before the cursor
    fn text_before_cursor(&self) -> String {
        self.text.chars()
            .take(self.cursor_position as usize)
            .collect::<String>()
    }

    /// Returns the text after the cursor
    fn text_after_cursor(&self) -> String {
        self.text.chars()
            .skip(self.cursor_position as usize)
            .collect::<String>()
    }

    /// Returns an index relative to the cursor position
    /// pointing to the start of the previous word. Return 0 if nothing was found.
    // TODO: replace return type with Option<i32>
    // TODO: consider returning unsigned num data type
    fn find_start_of_previous_word(&self) -> i32 {
        self.text_before_cursor()
            .rfind(' ')
            .map(|c| c + 1)
            .unwrap_or(0) as i32
    }

    /// Is almost the same as [find_start_of_previous_word].
    /// The only difference is to ignore contiguous spaces.
    // TODO: replace return type with Option<i32>
    // TODO: consider returning unsigned num data type
    fn find_start_of_previous_word_with_space(&self) -> i32 {
        let end = self.text_before_cursor()
            .rfind(|c| c != ' ');
        if let None = end {
            return 0;
        }
        let start = self.text_before_cursor()
            .split_at(end.unwrap())
            .0
            .rfind(' ');
        match start {
            None => 0,
            Some(start) => (start + 1) as i32
        }
    }

    /// Is almost the same as [find_start_of_previous_word](Document::find_start_of_previous_word).
    /// But this can specify Separator. Return 0 if nothing was found.
    // TODO: replace return type with Option<i32>
    // TODO: consider returning unsigned num data type
    fn find_start_of_previous_word_until_separator<S: AsRef<str>>(&self, sep: S) -> i32 {
        let sep = sep.as_ref();
        if sep.is_empty() {
            return self.find_start_of_previous_word();
        }

        self.text_before_cursor()
            .rfind(|c| sep.contains(c))
            .map(|c| c + 1)
            .unwrap_or(0) as i32
    }

    /// Is almost the same as find_start_of_previous_word_with_space.
    /// But this can specify Separator. Return 0 if nothing was found.
    fn find_start_of_previous_word_until_separator_ignore_next_to_cursor<S: AsRef<str>>(&self, sep: S) -> i32 {
        let sep = sep.as_ref();
        if sep.is_empty() {
            return self.find_start_of_previous_word_with_space();
        }
        let end = self.text_before_cursor()
            .rfind(|c| !sep.contains(c));
        match end {
            None => 0,
            Some(end) => {
                let start = self.text_before_cursor()
                    .split_at(end)
                    .0
                    .rfind(|c| sep.contains(c));
                match start {
                    None => 0,
                    Some(start) => (start + 1) as i32
                }
            }
        }
    }

    /// Returns an index relative to the cursor position.
    /// pointing to the end of the current word. Return 0 if nothing was found.
    // TODO: ported code, but doc comment seems outdated? https://github.com/c-bata/go-prompt/blob/82a912274504477990ecf7c852eebb7c85291772/document.go#L191
    fn find_end_of_current_word(&self) -> i32 {
        self.text_after_cursor()
            .find(' ')
            .unwrap_or(self.text_after_cursor().len()) as i32
    }

    /// Is almost the same as [find_end_of_current_word].
    /// The only difference is to ignore contiguous spaces.
    fn find_end_of_current_word_with_space(&self) -> i32 {
        let start = self.text_after_cursor()
            .find(|c| c != ' ');
        match start {
            None => self.text_after_cursor().len() as i32,
            Some(start) => {
                let end = self.text_after_cursor()
                    .split_at(start).1
                    .find(' ');
                match end {
                    None => self.text_after_cursor().len() as i32,
                    Some(end) => (start + end) as i32
                }
            }
        }
    }

    /// Is almost the same as [find_end_of_current_word].
    /// But this can specify Separator. Return 0 if nothing was found.
    fn find_end_of_current_word_until_separator<S: AsRef<str>>(&self, sep: S) -> i32 {
        let sep = sep.as_ref();
        if sep.is_empty() {
            self.find_end_of_current_word()
        } else {
            self.text_after_cursor()
                .find(|c| sep.contains(c))
                .unwrap_or(self.text_after_cursor().len()) as i32
        }
    }

    /// Is almost the same as [find_end_of_current_word_with_space].
    /// But this can specify Separator. Return 0 if nothing was found.
    fn find_end_of_current_word_until_separator_ignore_next_to_cursor<S: AsRef<str>>(&self, sep: S) -> i32 {
        let sep = sep.as_ref();
        if sep.is_empty() {
            self.find_end_of_current_word_with_space()
        } else {
            let start = self.text_after_cursor()
                .find(|c| !sep.contains(c));
            match start {
                None => self.text_after_cursor().len() as i32,
                Some(start) => {
                    let end = self.text_after_cursor()
                        .split_at(start).1
                        .find(|c| sep.contains(c));
                    match end {
                        None => self.text_after_cursor().len() as i32,
                        Some(end) => (start + end) as i32
                    }
                }
            }
        }
    }

    ///Returns the word before the cursor.
    /// If we have whitespace before the cursor this returns an empty string.
    fn get_word_before_cursor(&self) -> String {
        self.text_before_cursor()
            .split_at(self.find_start_of_previous_word() as usize).1
            .to_string()
    }

    /// Returns the word after the cursor.
    /// If we have whitespace after the cursor this returns an empty string.
    fn get_word_after_cursor(&self) -> String {
        self.text_after_cursor()
            .split_at(self.find_end_of_current_word() as usize).0
            .to_string()
    }

    /// Returns the word before the cursor.
    /// Unlike [get_word_before_cursor], it returns string containing space
    fn get_word_before_cursor_with_space(&self) -> String {
        self.text_before_cursor()
            .split_at(self.find_start_of_previous_word_with_space() as usize).1
            .to_string()
    }

    /// Returns the word after the cursor.
    /// Unlike [get_word_after_cursor], it returns string containing space
    fn get_word_after_cursor_with_space(&self) -> String {
        self.text_after_cursor()
            .split_at(self.find_end_of_current_word_with_space() as usize).0
            .to_string()
    }

    /// Returns the text before the cursor until next separator.
    fn get_word_before_cursor_until_separator<S: AsRef<str>>(&self, sep: S) -> String {
        self.text_before_cursor().split_at(self.find_start_of_previous_word_until_separator(sep) as usize).1
            .to_string()
    }

    /// Returns the text after the cursor until next separator.
    fn get_word_after_cursor_until_separator<S: AsRef<str>>(&self, sep: S) -> String {
        self.text_after_cursor().split_at(self.find_end_of_current_word_until_separator(sep) as usize).0
            .to_string()
    }

    /// Returns the word before the cursor.
    /// Unlike [get_word_before_cursor], it returns string containing space
    fn get_word_before_cursor_until_separator_ignore_next_to_cursor<S: AsRef<str>>(&self, sep: S) -> String {
        self.text_before_cursor().split_at(self.find_start_of_previous_word_until_separator_ignore_next_to_cursor(sep) as usize).1.to_string()
    }

    /// Returns the word after the cursor.
    /// Unlike [get_word_after_cursor], it returns string containing space
    fn get_word_after_cursor_until_separator_ignore_next_to_cursor<S: AsRef<str>>(&self, sep: S) -> String {
        self.text_after_cursor().split_at(self.find_end_of_current_word_until_separator_ignore_next_to_cursor(sep) as usize).0.to_string()
    }

    /// Returns the text from the start of the line until the cursor.
    fn current_line_before_cursor(&self) -> String {
        self.text_before_cursor().split('\n')
            .last()
            .expect("expected at least one substring")
            .to_string()
    }

    /// Returns the text from the cursor until the end of the line.
    fn current_line_after_cursor(&self) -> String {
        self.text_after_cursor().split('\n').take(1).collect::<String>()
    }

    /// Return the text on the line where the cursor is. (when the input
    /// consists of just one line, it equals `text`.
    fn current_line(&self) -> String {
        self.current_line_before_cursor() + self.current_line_after_cursor().as_str()
    }

    /// Returns a Vec of all the lines.
    // TODO: do we have to map to String?
    fn lines(&self) -> Vec<String> {
        self.text.split('\n').map(|s| s.to_string()).collect::<Vec<String>>()
    }

    /// Return the number of lines in this document. If the document ends
    /// with a trailing \n, that counts as the beginning of a new line.
    fn line_count(&self) -> usize {
        self.lines().len()
    }

    /// Array pointing to the start indexes of all the lines.
    fn line_start_indexes(&self) -> Vec<usize> {
        // TODO: Cache, because this is often reused.
        // (If it is used, it's often used many times.
        // And this has to be fast for editing big documents!)
        let lc = self.line_count();
        let lengths = self.lines()
            .into_iter()
            .map(|l| l.len())
            .collect::<Vec<_>>();

        let mut indexes = Vec::with_capacity(lc + 1);
        indexes.push(0); // https://github.com/jonathanslenders/python-prompt-toolkit/blob/master/prompt_toolkit/document.py#L189
        let mut pos = 0;
        for l in lengths {
            pos += l + 1;
            indexes.push(pos);
        }
        if lc > 1 {
            // Pop the last item. (This is not a new line.)
            indexes.pop().expect("expected to be able to pop last index");
        }
        indexes
    }

    /// For the index of a character at a certain line, calculate the index of
    /// the first character on that line.
    fn find_line_start_index(&self, index: usize) -> (usize, usize) {
        let indexes = self.line_start_indexes();
        let pos = bisect::right(&indexes, index) - 1;
        (pos, indexes[pos])
    }
}

mod bisect {
    use std::cmp::Ordering;

    pub fn right(a: &[usize], v: usize) -> usize {
        bisect_right_range(a, v, 0, a.len())
    }

    fn bisect_right_range(a: &[usize], v: usize, lo: usize, hi: usize) -> usize {
        let s = &a[lo..hi];
        s.partition_point(|&probe| {
            probe < v
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_bisect_right() {
            let input = vec![1, 2, 3, 3, 3, 6, 7];

            let r = right(&input, 0);
            assert_eq!(0, r, "number 0 should inserted at 0 position, but got {}", r);
            let r = right(&input, 4);
            assert_eq!(5, r, "number 4 should inserted at 5 position, but got {}", r);
            let r = right(&input, 8);
            assert_eq!(7, r, "number 8 should inserted at 7 position, but got {}", r);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_cursor_position() {
        assert_eq!(2, Document {
            text: "hello".to_string(),
            cursor_position: 2,
            ..Default::default()
        }.display_cursor_position());
        assert_eq!(4, Document {
            text: "こんにちは".to_string(),
            cursor_position: 2,
            ..Default::default()
        }.display_cursor_position());
        // If you're facing test failure on this test case and your terminal is iTerm2,
        // please check 'Profile -> Text' configuration. 'Use Unicode version 9 widths'
        // must be checked.
        // https://github.com/c-bata/go-prompt/pull/99
        assert_eq!(3, Document {
            text: "Добрый день".to_string(),
            cursor_position: 3,
            ..Default::default()
        }.display_cursor_position());
    }

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

    #[test]
    fn test_text_before_cursor() {
        assert_eq!("line 1\nlin", Document {
            text: "line 1\nline 2\nline 3\nline 4\n".to_string(),
            cursor_position: ("line 1\n".to_string() + "lin").chars().count() as i32,
            ..Default::default()
        }.text_before_cursor());
        assert_eq!("あいうえお\nかき", Document {
            text: "あいうえお\nかきくけこ\nさしすせそ\nたちつてと\n".to_string(),
            cursor_position: 8,
            ..Default::default()
        }.text_before_cursor());
        assert_eq!("Добрый\nде", Document {
            text: "Добрый\nдень\nДобрый день".to_string(),
            cursor_position: 9,
            ..Default::default()
        }.text_before_cursor());
    }

    #[test]
    fn test_text_after_cursor() {
        assert_eq!("e 2\nline 3\nline 4\n", Document {
            text: "line 1\nline 2\nline 3\nline 4\n".to_string(),
            cursor_position: ("line 1\n".to_string() + "lin").chars().count() as i32,
            ..Default::default()
        }.text_after_cursor());
        assert_eq!("くけこ\nさしすせそ\nたちつてと\n", Document {
            text: "あいうえお\nかきくけこ\nさしすせそ\nたちつてと\n".to_string(),
            cursor_position: 8,
            ..Default::default()
        }.text_after_cursor());
        assert_eq!("нь\nДобрый день", Document {
            text: "Добрый\nдень\nДобрый день".to_string(),
            cursor_position: 9,
            ..Default::default()
        }.text_after_cursor());
    }

    // TODO: consider using macros for testcases
    #[test]
    fn test_find_start_of_previous_word() {
        assert_eq!("apple ".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple bana".len() as i32,
            ..Default::default()
        }.find_start_of_previous_word());
        assert_eq!("apple ".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple bana".len() as i32,
            ..Default::default()
        }.find_start_of_previous_word_until_separator(""));

        assert_eq!("apply -f ./file/".len() as i32, Document {
            text: "apply -f ./file/foo.json".to_string(),
            cursor_position: "apply -f ./file/foo.json".len() as i32,
            ..Default::default()
        }.find_start_of_previous_word_until_separator(" /"));

        assert_eq!("apple ".len() as i32, Document {
            text: "apple ".to_string(),
            cursor_position: "apple ".len() as i32,
            ..Default::default()
        }.find_start_of_previous_word());
        assert_eq!("apple ".len() as i32, Document {
            text: "apple ".to_string(),
            cursor_position: "apple ".len() as i32,
            ..Default::default()
        }.find_start_of_previous_word_until_separator(""));

        assert_eq!("apply -f ./".len() as i32, Document {
            text: "apply -f ./file/foo.json".to_string(),
            cursor_position: "apply -f ./".len() as i32,
            ..Default::default()
        }.find_start_of_previous_word_until_separator(" /"));

        assert_eq!("あいうえお ".len() as i32, Document {
            text: "あいうえお かきくけこ さしすせそ".to_string(),
            cursor_position: 8,
            ..Default::default()
        }.find_start_of_previous_word());
        assert_eq!("あいうえお ".len() as i32, Document {
            text: "あいうえお かきくけこ さしすせそ".to_string(),
            cursor_position: 8,
            ..Default::default()
        }.find_start_of_previous_word_until_separator(""));

        assert_eq!("Добрый ".len() as i32, Document {
            text: "Добрый день Добрый день".to_string(),
            cursor_position: 9,
            ..Default::default()
        }.find_start_of_previous_word());
        assert_eq!("Добрый ".len() as i32, Document {
            text: "Добрый день Добрый день".to_string(),
            cursor_position: 9,
            ..Default::default()
        }.find_start_of_previous_word_until_separator(""));
    }

    #[test]
    fn test_find_start_of_previous_word_with_space() {
        assert_eq!("apple ".len() as i32, Document {
            text: "apple bana ".to_string(),
            cursor_position: "apple bana ".len() as i32,
            ..Default::default()
        }.find_start_of_previous_word_with_space());
        assert_eq!("apple ".len() as i32, Document {
            text: "apple bana ".to_string(),
            cursor_position: "apple bana ".len() as i32,
            ..Default::default()
        }.find_start_of_previous_word_until_separator_ignore_next_to_cursor(""));

        assert_eq!("apply -f /file/".len() as i32, Document {
            text: "apply -f /file/foo/".to_string(),
            cursor_position: "apply -f /file/foo/".len() as i32,
            ..Default::default()
        }.find_start_of_previous_word_until_separator_ignore_next_to_cursor(" /"));

        assert_eq!("".len() as i32, Document {
            text: "apple ".to_string(),
            cursor_position: "apple ".len() as i32,
            ..Default::default()
        }.find_start_of_previous_word_with_space());
        assert_eq!("".len() as i32, Document {
            text: "apple ".to_string(),
            cursor_position: "apple ".len() as i32,
            ..Default::default()
        }.find_start_of_previous_word_until_separator_ignore_next_to_cursor(""));

        assert_eq!("".len() as i32, Document {
            text: "file/".to_string(),
            cursor_position: "file/".len() as i32,
            ..Default::default()
        }.find_start_of_previous_word_until_separator_ignore_next_to_cursor(" /"));

        assert_eq!("あいうえお ".len() as i32, Document {
            text: "あいうえお かきくけこ ".to_string(),
            cursor_position: 12,
            ..Default::default()
        }.find_start_of_previous_word_with_space());
        assert_eq!("あいうえお ".len() as i32, Document {
            text: "あいうえお かきくけこ ".to_string(),
            cursor_position: 12,
            ..Default::default()
        }.find_start_of_previous_word_until_separator_ignore_next_to_cursor(""));

        assert_eq!("Добрый ".len() as i32, Document {
            text: "Добрый день ".to_string(),
            cursor_position: 12,
            ..Default::default()
        }.find_start_of_previous_word_with_space());
        assert_eq!("Добрый ".len() as i32, Document {
            text: "Добрый день ".to_string(),
            cursor_position: 12,
            ..Default::default()
        }.find_start_of_previous_word_until_separator_ignore_next_to_cursor(""));
    }

    #[test]
    fn test_find_end_of_current_word() {
        assert_eq!("".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple bana".len() as i32,
            ..Default::default()
        }.find_end_of_current_word());
        assert_eq!("".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple bana".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_until_separator(""));

        assert_eq!("bana".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple ".len() as i32,
            ..Default::default()
        }.find_end_of_current_word());
        assert_eq!("bana".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple ".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_until_separator(""));

        assert_eq!("file".len() as i32, Document {
            text: "apply -f ./file/foo.json".to_string(),
            cursor_position: "apply -f ./".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_until_separator(" /"));

        assert_eq!("".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple".len() as i32,
            ..Default::default()
        }.find_end_of_current_word());
        assert_eq!("".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_until_separator(""));

        assert_eq!("".len() as i32, Document {
            text: "apply -f ./file/foo.json".to_string(),
            cursor_position: "apply -f .".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_until_separator(" /"));

        assert_eq!("ple".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "ap".len() as i32,
            ..Default::default()
        }.find_end_of_current_word());
        assert_eq!("ple".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "ap".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_until_separator(""));

        // りん(cursor)ご ばなな
        assert_eq!("ご".len() as i32, Document {
            text: "りんご ばなな".to_string(),
            cursor_position: 2,
            ..Default::default()
        }.find_end_of_current_word());
        assert_eq!("ご".len() as i32, Document {
            text: "りんご ばなな".to_string(),
            cursor_position: 2,
            ..Default::default()
        }.find_end_of_current_word_until_separator(""));

        assert_eq!(0, Document {
            text: "りんご ばなな".to_string(),
            cursor_position: 3,
            ..Default::default()
        }.find_end_of_current_word());
        assert_eq!(0, Document {
            text: "りんご ばなな".to_string(),
            cursor_position: 3,
            ..Default::default()
        }.find_end_of_current_word_until_separator(""));

        // Доб(cursor)рый день
        assert_eq!("рый".len() as i32, Document {
            text: "Добрый день".to_string(),
            cursor_position: 3,
            ..Default::default()
        }.find_end_of_current_word());
        assert_eq!("рый".len() as i32, Document {
            text: "Добрый день".to_string(),
            cursor_position: 3,
            ..Default::default()
        }.find_end_of_current_word_until_separator(""));
    }

    #[test]
    fn test_find_end_of_current_word_with_space() {
        assert_eq!("".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple bana".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_with_space());
        assert_eq!("".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple bana".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_until_separator_ignore_next_to_cursor(""));

        assert_eq!("bana".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple ".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_with_space());
        assert_eq!("bana".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple ".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_until_separator_ignore_next_to_cursor(""));

        assert_eq!("file".len() as i32, Document {
            text: "apply -f /file/foo.json".to_string(),
            cursor_position: "apply -f /".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_until_separator_ignore_next_to_cursor(" /"));

        assert_eq!(" bana".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_with_space());
        assert_eq!(" bana".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "apple".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_until_separator_ignore_next_to_cursor(""));

        assert_eq!("/to".len() as i32, Document {
            text: "apply -f /path/to".to_string(),
            cursor_position: "apply -f /path".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_until_separator_ignore_next_to_cursor(" /"));

        assert_eq!("ple".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "ap".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_with_space());
        assert_eq!("ple".len() as i32, Document {
            text: "apple bana".to_string(),
            cursor_position: "ap".len() as i32,
            ..Default::default()
        }.find_end_of_current_word_until_separator_ignore_next_to_cursor(""));

        assert_eq!("かきくけこ".len() as i32, Document {
            text: "あいうえお かきくけこ".to_string(),
            cursor_position: 6,
            ..Default::default()
        }.find_end_of_current_word_with_space());
        assert_eq!("かきくけこ".len() as i32, Document {
            text: "あいうえお かきくけこ".to_string(),
            cursor_position: 6,
            ..Default::default()
        }.find_end_of_current_word_until_separator_ignore_next_to_cursor(""));

        assert_eq!(" かきくけこ".len() as i32, Document {
            text: "あいうえお かきくけこ".to_string(),
            cursor_position: 5,
            ..Default::default()
        }.find_end_of_current_word_with_space());
        assert_eq!(" かきくけこ".len() as i32, Document {
            text: "あいうえお かきくけこ".to_string(),
            cursor_position: 5,
            ..Default::default()
        }.find_end_of_current_word_until_separator_ignore_next_to_cursor(""));

        assert_eq!(" день".len() as i32, Document {
            text: "Добрый день".to_string(),
            cursor_position: 6,
            ..Default::default()
        }.find_end_of_current_word_with_space());
        assert_eq!(" день".len() as i32, Document {
            text: "Добрый день".to_string(),
            cursor_position: 6,
            ..Default::default()
        }.find_end_of_current_word_until_separator_ignore_next_to_cursor(""));
    }

    #[test]
    fn test_get_word_after_cursor() {
        assert_eq!("", Document {
            text: "apple bana".to_string(),
            cursor_position: "apple bana".len() as i32,
            ..Default::default()
        }.get_word_after_cursor());
        assert_eq!("", Document {
            text: "apple bana".to_string(),
            cursor_position: "apple bana".len() as i32,
            ..Default::default()
        }.get_word_after_cursor_until_separator(""));

        assert_eq!("le", Document {
            text: "apply -f ./file/foo.json".to_string(),
            cursor_position: "apply -f ./fi".len() as i32,
            ..Default::default()
        }.get_word_after_cursor_until_separator("/"));

        assert_eq!("bana", Document {
            text: "apple bana".to_string(),
            cursor_position: "apple ".len() as i32,
            ..Default::default()
        }.get_word_after_cursor());
        assert_eq!("bana", Document {
            text: "apple bana".to_string(),
            cursor_position: "apple ".len() as i32,
            ..Default::default()
        }.get_word_after_cursor_until_separator(""));

        assert_eq!("", Document {
            text: "apple bana".to_string(),
            cursor_position: "apple".len() as i32,
            ..Default::default()
        }.get_word_after_cursor());
        assert_eq!("", Document {
            text: "apple bana".to_string(),
            cursor_position: "apple".len() as i32,
            ..Default::default()
        }.get_word_after_cursor_until_separator(""));

        assert_eq!("", Document {
            text: "apply -f ./file/foo.json".to_string(),
            cursor_position: "apply -f .".len() as i32,
            ..Default::default()
        }.get_word_after_cursor_until_separator(" /"));

        assert_eq!("ple", Document {
            text: "apple bana".to_string(),
            cursor_position: "ap".len() as i32,
            ..Default::default()
        }.get_word_after_cursor());
        assert_eq!("ple", Document {
            text: "apple bana".to_string(),
            cursor_position: "ap".len() as i32,
            ..Default::default()
        }.get_word_after_cursor_until_separator(""));

        assert_eq!("くけこ", Document {
            text: "あいうえお かきくけこ さしすせそ".to_string(),
            cursor_position: 8,
            ..Default::default()
        }.get_word_after_cursor());
        assert_eq!("くけこ", Document {
            text: "あいうえお かきくけこ さしすせそ".to_string(),
            cursor_position: 8,
            ..Default::default()
        }.get_word_after_cursor_until_separator(""));

        assert_eq!("нь", Document {
            text: "Добрый день Добрый день".to_string(),
            cursor_position: 9,
            ..Default::default()
        }.get_word_after_cursor());
        assert_eq!("нь", Document {
            text: "Добрый день Добрый день".to_string(),
            cursor_position: 9,
            ..Default::default()
        }.get_word_after_cursor_until_separator(""));
    }

    #[test]
    fn test_current_line_before_cursor() {
        assert_eq!("lin", Document {
            text: "line 1\nline 2\nline 3\nline 4\n".to_string(),
            cursor_position: "line 1\nlin".len() as i32,
            ..Default::default()
        }.current_line_before_cursor());
    }

    #[test]
    fn test_current_line_after_cursor() {
        assert_eq!("e 2", Document {
            text: "line 1\nline 2\nline 3\nline 4\n".to_string(),
            cursor_position: "line 1\nlin".len() as i32,
            ..Default::default()
        }.current_line_after_cursor());
    }

    #[test]
    fn test_current_line() {
        assert_eq!("line 2", Document {
            text: "line 1\nline 2\nline 3\nline 4\n".to_string(),
            cursor_position: "line 1\nlin".len() as i32,
            ..Default::default()
        }.current_line());
    }
}