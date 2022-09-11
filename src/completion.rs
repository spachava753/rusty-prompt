use std::process::id;
use std::thread::spawn;

const SHORTEN_SUFFIX: &str = "...";
const LEFT_PREFIX: &str = " ";
const LEFT_SUFFIX: &str = " ";
const RIGHT_PREFIX: &str = " ";
const RIGHT_SUFFIX: &str = " ";

#[derive(Debug, Eq, PartialEq)]
pub struct Suggestion {
    text: String,
    description: String,
}

impl Suggestion {
    // TODO: figure out a way to accept String and &str?
    pub fn new(text: String, description: String) -> Self {
        Self {
            text,
            description,
        }
    }

    pub fn with_title(text: String) -> Self {
        Self {
            text,
            description: "".to_string(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

trait Completer {
    // TODO: maybe better to do `&mut self`
    fn complete(&self, input: &str) -> Vec<Suggestion>;
}

#[derive(Default)]
struct CompletionManager<'a, C: Completer + Default> {
    selected: i32,
    tmp: Vec<Suggestion>,
    max: usize,
    completer: C,
    vertical_scroll: isize,
    word_separator: &'a str,
    show_at_start: bool,
}

impl<'a, C: Completer + Default> CompletionManager<'a, C> {
    fn new(completer: C, max: usize) -> Self {
        Self {
            completer,
            selected: -1,
            max,
            vertical_scroll: 0,
            ..Default::default()
        }
    }

    fn get_suggestions(&self) -> &[Suggestion] {
        &self.tmp
    }

    fn update_suggestions(&mut self, input: &str) {
        self.tmp = self.completer.complete(input);
    }

    fn update(&mut self) {
        let max = self.max.min(self.tmp.len());

        if self.selected > -1 && self.selected as usize >= self.tmp.len() {
            self.reset()
        } else if self.selected < -1 {
            self.selected = self.tmp.len() as i32 - 1;
            self.vertical_scroll = (self.tmp.len() - max) as isize;
        }
    }

    fn reset(&mut self) {
        self.selected = -1;
        self.vertical_scroll = 0;
        self.update_suggestions("");
    }

    fn previous(&mut self) {
        if self.vertical_scroll == self.selected as isize && self.selected > 0 {
            self.vertical_scroll -= 1;
        }
        self.selected -= 1;
        self.update();
    }

    fn next(&mut self) {
        if self.vertical_scroll + self.max as isize - 1 == self.selected as isize {
            self.vertical_scroll += 1;
        }
        self.selected += 1;
        self.update();
    }

    fn completing(&self) -> bool {
        self.selected != -1
    }
}

fn delete_break_line_characters(s: &str) -> String {
    let s = s.replace("\n", "");
    let s = s.replace("\r", "");
    s
}

fn format_texts(o: &[&str], max: usize, prefix: &str, suffix: &str) -> (Vec<String>, usize) {
    let mut n = vec!["".to_string(); o.len()];

    let len_prefix = prefix.len();
    let len_suffix = suffix.len();
    let len_shorten = SHORTEN_SUFFIX.len();
    let min = len_prefix + len_suffix + len_shorten;

    let mut width = o.iter()
        .map(|s| delete_break_line_characters(s).len())
        .max()
        .unwrap_or(0);

    if width == 0 {
        return (n, width);
    }

    if min >= max {
        return (n, 0);
    }

    let width = if len_prefix + width + len_suffix > max {
        max - len_prefix - len_suffix
    } else {
        width
    };

    for (idx, &i) in o.iter().enumerate() {
        let x = i.len();
        if x <= width {
            let spaces = " ".repeat(width - x);
            n[idx] = (prefix.to_string() + i + &spaces + suffix);
        } else if x > width {
            let mut i = i.clone();
            let mut i = i.to_string();
            i.truncate(width - SHORTEN_SUFFIX.len());
            let mut x = i + SHORTEN_SUFFIX;
            if x.len() < width {
                x = format!("{:count$}", x, count = width - x.len());
            }
            n[idx] = (prefix.to_string() + &x + suffix);
        }
    }

    return (n, len_prefix + width + len_suffix);
}

// TODO: convert this to return Result<(Vec<Suggestion>, usize)>. Use eyre?
fn format_suggestions(suggestions: &[Suggestion], max: usize) -> (Vec<Suggestion>, usize) {
    let left = suggestions.iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<&str>>();
    let right = suggestions.iter()
        .map(|s| s.description.as_str())
        .collect::<Vec<&str>>();

    let (left, left_width) = format_texts(
        &left,
        max,
        LEFT_PREFIX,
        LEFT_SUFFIX,
    );
    if left_width == 0 {
        return (vec![], 0);
    }
    let (right, right_width) = if max > left_width {
        format_texts(
            &right,
            max - left_width,
            RIGHT_PREFIX,
            RIGHT_SUFFIX,
        )
    } else {
        (vec!["".to_string(); right.len()], 0)
    };

    let new_suggestions = left.into_iter()
        .zip(right)
        .map(|(text, desc)| Suggestion::new(text, desc))
        .collect::<Vec<Suggestion>>();

    return (new_suggestions, left_width + right_width);
}

#[cfg(test)]
mod tests {
    use std::ops::Add;
    use super::*;

    fn compare_format_suggestions(
        suggestions: Vec<Suggestion>,
        width: usize,
        expected: Vec<Suggestion>,
        ex_width: usize,
    ) {
        if width != ex_width {
            panic!("got: {}, expected: {}", width, ex_width);
        }

        if suggestions.len() != expected.len() {
            panic!("got len: {}, expected: {}", suggestions.len(), expected.len());
        }

        suggestions.into_iter().zip(expected).for_each(|(got, want)| {
            if got != want {
                panic!("got: {:?}, want: {:?}", got, want);
            }
        });
    }

    #[test]
    fn test_format_suggestions_title() {
        let input = vec![
            Suggestion::with_title("foo".to_string()),
            Suggestion::with_title("bar".to_string()),
            Suggestion::with_title("fuga".to_string()),
        ];
        let expected = vec![
            Suggestion::with_title(" foo  ".to_string()),
            Suggestion::with_title(" bar  ".to_string()),
            Suggestion::with_title(" fuga ".to_string()),
        ];
        let max = 100;
        let ex_wdith = 6;
        let (suggestions, width) = format_suggestions(&input, max);
        compare_format_suggestions(suggestions, width, expected, ex_wdith);
    }

    #[test]
    fn test_format_suggestions_test_scenario() {
        let input = vec![
            Suggestion::new("apple".to_string(), "This is apple.".to_string()),
            Suggestion::new("banana".to_string(), "This is banana.".to_string()),
            Suggestion::new("coconut".to_string(), "This is coconut.".to_string()),
        ];
        let expected = vec![
            Suggestion::new(" apple   ".to_string(), " This is apple.   ".to_string()),
            Suggestion::new(" banana  ".to_string(), " This is banana.  ".to_string()),
            Suggestion::new(" coconut ".to_string(), " This is coconut. ".to_string()),
        ];
        let max = 100;
        let ex_wdith = " apple   ".to_string().add(" This is apple.   ").len();
        let (suggestions, width) = format_suggestions(&input, max);
        compare_format_suggestions(suggestions, width, expected, ex_wdith);
    }

    #[test]
    fn test_format_suggestions_small_width() {
        let input = vec![
            Suggestion::with_title("This is apple.".to_string()),
            Suggestion::with_title("This is banana.".to_string()),
            Suggestion::with_title("This is coconut.".to_string()),
        ];
        let expected = vec![
            Suggestion::with_title(" Thi... ".to_string()),
            Suggestion::with_title(" Thi... ".to_string()),
            Suggestion::with_title(" Thi... ".to_string()),
        ];
        let max = 8;
        let ex_wdith = 8;
        let (suggestions, width) = format_suggestions(&input, max);
        compare_format_suggestions(suggestions, width, expected, ex_wdith);
    }

    #[test]
    fn test_format_suggestions_too_small_max() {
        let input = vec![
            Suggestion::with_title("This is apple.".to_string()),
            Suggestion::with_title("This is banana.".to_string()),
            Suggestion::with_title("This is coconut.".to_string()),
        ];
        let expected = Vec::new();
        let max = 3;
        let ex_wdith = 0;
        let (suggestions, width) = format_suggestions(&input, max);
        compare_format_suggestions(suggestions, width, expected, ex_wdith);
    }

    #[test]
    fn test_format_suggestions_big_description() {
        let input = vec![
            Suggestion::new("--all-namespaces".to_string(), "-------------------------------------------------------------------------------------------------------------------------------------------".to_string()),
            Suggestion::new("--allow-missing-template-keys".to_string(), "-----------------------------------------------------------------------------------------------------------------------------------------------".to_string()),
            Suggestion::new("--export".to_string(), "----------------------------------------------------------------------------------------------------------".to_string()),
            Suggestion::new("-f".to_string(), "-----------------------------------------------------------------------------------".to_string()),
            Suggestion::new("--filename".to_string(), "-----------------------------------------------------------------------------------".to_string()),
            Suggestion::new("--include-extended-apis".to_string(), "------------------------------------------------------------------------------------".to_string()),
        ];
        let expected = vec![
            Suggestion::new(" --all-namespaces              ".to_string(), " --------------... ".to_string()),
            Suggestion::new(" --allow-missing-template-keys ".to_string(), " --------------... ".to_string()),
            Suggestion::new(" --export                      ".to_string(), " --------------... ".to_string()),
            Suggestion::new(" -f                            ".to_string(), " --------------... ".to_string()),
            Suggestion::new(" --filename                    ".to_string(), " --------------... ".to_string()),
            Suggestion::new(" --include-extended-apis       ".to_string(), " --------------... ".to_string()),
        ];
        let max = 50;
        let ex_wdith = expected.last().unwrap().text.len() +
            expected.last().unwrap().description.len();
        let (suggestions, width) = format_suggestions(&input, max);
        compare_format_suggestions(suggestions, width, expected, ex_wdith);
    }

    #[test]
    fn test_format_suggestions_example_scenario() {
        let input = vec![
            Suggestion::new("--all-namespaces".to_string(), "If present, list the requested object(s) across all namespaces. Namespace in current context is ignored even if specified with --namespace.".to_string()),
            Suggestion::new("--allow-missing-template-keys".to_string(), "If true, ignore any errors in templates when a field or map key is missing in the template. Only applies to golang and jsonpath output formats.".to_string()),
            Suggestion::new("--export".to_string(), "If true, use 'export' for the resources.  Exported resources are stripped of cluster-specific information.".to_string()),
            Suggestion::new("-f".to_string(), "Filename, directory, or URL to files identifying the resource to get from a server.".to_string()),
            Suggestion::new("--filename".to_string(), "Filename, directory, or URL to files identifying the resource to get from a server.".to_string()),
            Suggestion::new("--include-extended-apis".to_string(), "If true, include definitions of new APIs via calls to the API server. [default true]".to_string()),
        ];
        let expected = vec![
            Suggestion::new(" --all-namespaces              ".to_string(), " If present, list the requested object(s) across all namespaces. Namespace in current context is ignored even if specified with --namespace.     ".to_string()),
            Suggestion::new(" --allow-missing-template-keys ".to_string(), " If true, ignore any errors in templates when a field or map key is missing in the template. Only applies to golang and jsonpath output formats. ".to_string()),
            Suggestion::new(" --export                      ".to_string(), " If true, use 'export' for the resources.  Exported resources are stripped of cluster-specific information.                                      ".to_string()),
            Suggestion::new(" -f                            ".to_string(), " Filename, directory, or URL to files identifying the resource to get from a server.                                                             ".to_string()),
            Suggestion::new(" --filename                    ".to_string(), " Filename, directory, or URL to files identifying the resource to get from a server.                                                             ".to_string()),
            Suggestion::new(" --include-extended-apis       ".to_string(), " If true, include definitions of new APIs via calls to the API server. [default true]                                                            ".to_string()),
        ];
        let max = 500;
        let ex_wdith = expected.last().unwrap().text.len() +
            expected.last().unwrap().description.len();
        let (suggestions, width) = format_suggestions(&input, max);
        compare_format_suggestions(suggestions, width, expected, ex_wdith);
    }

    fn compare_format_text(input: Vec<String>, width: usize, expected: Vec<&str>, ex_width: usize) {
        if width != ex_width {
            panic!("width got: {}, want: {}", width, ex_width);
        }

        if input.len() != expected.len() {
            panic!("len got: {}, want: {}", input.len(), expected.len());
        }

        if !input.eq(&expected) {
            panic!("result got: {:?}, want: {:?}", input, expected);
        }
    }

    #[test]
    fn test_format_text_blank() {
        let input = vec!["", ""];
        let expected = vec!["", ""];
        let max = 10;
        let ex_width = 0;
        let (actual, width) = format_texts(&input, max, " ", " ");
        compare_format_text(actual, width, expected, ex_width);
    }

    #[test]
    fn test_format_text_small_max() {
        let input = vec!["apple", "banana", "coconut"];
        let expected = vec!["", "", ""];
        let max = 2;
        let ex_width = 0;
        let (actual, width) = format_texts(&input, max, " ", " ");
        compare_format_text(actual, width, expected, ex_width);
    }

    #[test]
    fn test_format_text_small_max_2() {
        let input = vec!["apple", "banana", "coconut"];
        let expected = vec!["", "", ""];
        let max = (" ".to_string() + " " + SHORTEN_SUFFIX).len();
        let ex_width = 0;
        let (actual, width) = format_texts(&input, max, " ", " ");
        compare_format_text(actual, width, expected, ex_width);
    }

    #[test]
    fn test_format_text_example() {
        let input = vec!["apple", "banana", "coconut"];
        let expected = vec![" apple   ", " banana  ", " coconut "];
        let max = 100;
        let ex_width = expected.last().unwrap().len();
        let (actual, width) = format_texts(&input, max, " ", " ");
        compare_format_text(actual, width, expected, ex_width);
    }

    #[test]
    fn test_format_text_shorten() {
        let input = vec!["apple", "banana", "coconut"];
        let expected = vec![" a... ", " b... ", " c... "];
        let max = 6;
        let ex_width = expected.last().unwrap().len();
        let (actual, width) = format_texts(&input, max, " ", " ");
        compare_format_text(actual, width, expected, ex_width);
    }
}