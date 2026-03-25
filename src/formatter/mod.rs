mod align;
mod indent;
mod module_header;
mod util;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::config::FormatSettings;

use self::util::{spaces, split_line_comment};

static IF_KEYWORD_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bif\b\s*").unwrap());
static ELSE_KEYWORD_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\belse\b\s*").unwrap());
static NONBLOCKING_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*<=\s*").unwrap());
static OPEN_PAREN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(\s*").unwrap());
static CLOSE_PAREN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*\)").unwrap());
static OPEN_BRACKET_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[\s*").unwrap());
static CLOSE_BRACKET_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*\]").unwrap());

pub struct Formatter {
    settings: FormatSettings,
}

impl Formatter {
    pub fn new(settings: FormatSettings) -> Self {
        Self { settings }
    }

    pub fn format(&self, input: &str) -> String {
        let mut buffer = input
            .lines()
            .map(|line| line.trim().to_string())
            .collect::<Vec<_>>();

        if buffer.is_empty() {
            return String::new();
        }

        indent::apply(&mut buffer, &self.settings);
        module_header::apply(&mut buffer);

        decorate_non_block_comment_lines(&mut buffer, |line| {
            normalize_trailing_comments(line, &self.settings)
        });
        decorate_non_block_comment_lines(&mut buffer, |line| {
            normalize_if_spacing(line, &self.settings)
        });
        decorate_non_block_comment_lines(&mut buffer, |line| {
            normalize_blocking_assignment_spacing(line, &self.settings)
        });
        decorate_non_block_comment_lines(&mut buffer, |line| {
            normalize_nonblocking_assignment_spacing(line, &self.settings)
        });
        decorate_non_block_comment_lines(&mut buffer, |line| {
            normalize_parentheses_spacing(line, &self.settings)
        });
        decorate_non_block_comment_lines(&mut buffer, |line| {
            normalize_square_bracket_spacing(line, &self.settings)
        });

        if self.settings.align_blocking_assignments {
            align::align_blocking_assignments(&mut buffer);
        }
        if self.settings.align_no_blocking_assignments {
            align::align_no_blocking_assignments(&mut buffer);
        }
        if self.settings.align_line_comments {
            align::align_line_comments(&mut buffer);
        }

        let mut output = buffer.join("\n");
        output.push('\n');
        output
    }
}

fn decorate_non_block_comment_lines(
    buffer: &mut [String],
    mut decorate: impl FnMut(&str) -> String,
) {
    let mut in_block_comment = false;

    for line in buffer.iter_mut() {
        let trimmed = line.trim_start();

        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("/*") && !trimmed.contains("*/") {
            in_block_comment = true;
            continue;
        }

        *line = decorate(line);
    }
}

fn normalize_trailing_comments(line: &str, settings: &FormatSettings) -> String {
    let (code, comment) = split_line_comment(line);
    let Some(comment) = comment else {
        return line.to_string();
    };

    let comment_body = comment.trim_start_matches("//").trim_start();
    let before = spaces(settings.spaces_before_trailing_comments);
    let after = spaces(settings.spaces_after_trailing_comments);

    if code.trim().is_empty() {
        return format!("//{after}{comment_body}");
    }

    format!("{}{}//{}{}", code.trim_end(), before, after, comment_body)
}

fn normalize_if_spacing(line: &str, settings: &FormatSettings) -> String {
    let (code, comment) = split_line_comment(line);
    let gap = spaces(settings.spaces_before_if_statement);

    let updated = IF_KEYWORD_RE
        .replace_all(code, format!("if{gap}"))
        .into_owned();
    let updated = ELSE_KEYWORD_RE
        .replace_all(&updated, format!("else{gap}"))
        .into_owned();

    match comment {
        Some(comment) => format!("{updated}{comment}"),
        None => updated,
    }
}

fn normalize_nonblocking_assignment_spacing(line: &str, settings: &FormatSettings) -> String {
    let (code, comment) = split_line_comment(line);
    let gap = spaces(settings.spaces_no_blocking_assignment);
    let updated = NONBLOCKING_RE
        .replace_all(code, format!("{gap}<={gap}"))
        .into_owned();

    match comment {
        Some(comment) => format!("{updated}{comment}"),
        None => updated,
    }
}

fn normalize_blocking_assignment_spacing(line: &str, settings: &FormatSettings) -> String {
    let (code, comment) = split_line_comment(line);
    let updated = normalize_single_equals(code, settings.spaces_blocking_assignment);

    match comment {
        Some(comment) => format!("{updated}{comment}"),
        None => updated,
    }
}

fn normalize_parentheses_spacing(line: &str, settings: &FormatSettings) -> String {
    let (code, comment) = split_line_comment(line);

    let updated = if settings.spaces_in_parentheses {
        let opened = OPEN_PAREN_RE.replace_all(code, "( ").into_owned();
        CLOSE_PAREN_RE.replace_all(&opened, " )").into_owned()
    } else {
        let opened = OPEN_PAREN_RE.replace_all(code, "(").into_owned();
        CLOSE_PAREN_RE.replace_all(&opened, ")").into_owned()
    };

    match comment {
        Some(comment) => format!("{updated}{comment}"),
        None => updated,
    }
}

fn normalize_square_bracket_spacing(line: &str, settings: &FormatSettings) -> String {
    let (code, comment) = split_line_comment(line);

    let updated = if settings.spaces_in_square_brackets {
        let opened = OPEN_BRACKET_RE.replace_all(code, "[ ").into_owned();
        CLOSE_BRACKET_RE.replace_all(&opened, " ]").into_owned()
    } else {
        let opened = OPEN_BRACKET_RE.replace_all(code, "[").into_owned();
        CLOSE_BRACKET_RE.replace_all(&opened, "]").into_owned()
    };

    match comment {
        Some(comment) => format!("{updated}{comment}"),
        None => updated,
    }
}

fn normalize_single_equals(code: &str, space_count: usize) -> String {
    let gap = spaces(space_count);
    let bytes = code.as_bytes();
    let mut output = String::with_capacity(code.len() + 8);
    let mut index = 0;

    while index < bytes.len() {
        let current = bytes[index] as char;

        if current == '=' && is_blocking_assignment_operator(bytes, index) {
            while output.ends_with(' ') {
                output.pop();
            }
            output.push_str(&gap);
            output.push('=');
            index += 1;
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            output.push_str(&gap);
            continue;
        }

        output.push(current);
        index += 1;
    }

    output
}

fn is_blocking_assignment_operator(bytes: &[u8], index: usize) -> bool {
    let previous = previous_non_space(bytes, index);
    let next = next_non_space(bytes, index + 1);

    if next == Some(b'=') {
        return false;
    }

    !matches!(
        previous,
        Some(b'=')
            | Some(b'!')
            | Some(b'<')
            | Some(b'>')
            | Some(b'&')
            | Some(b'|')
            | Some(b'~')
            | Some(b'^')
            | Some(b'+')
            | Some(b'-')
            | Some(b'*')
            | Some(b'/')
    )
}

fn previous_non_space(bytes: &[u8], mut index: usize) -> Option<u8> {
    while index > 0 {
        index -= 1;
        if !bytes[index].is_ascii_whitespace() {
            return Some(bytes[index]);
        }
    }
    None
}

fn next_non_space(bytes: &[u8], mut index: usize) -> Option<u8> {
    while index < bytes.len() {
        if !bytes[index].is_ascii_whitespace() {
            return Some(bytes[index]);
        }
        index += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::config::FormatSettings;

    use super::Formatter;

    #[test]
    fn formatter_applies_indentation_and_nonblocking_spacing() {
        let input = "\
module foo;
always @(posedge clk)
if(load==1)
bitc<=0;
endmodule
";

        let expected = "\
module foo;
    always @(posedge clk)
        if (load==1)
            bitc <= 0;
endmodule
";

        let actual = Formatter::new(FormatSettings::default()).format(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn formatter_supports_parentheses_and_square_bracket_spacing() {
        let input = "\
module foo;
reg[addr] <= data;
if(load==1)
endmodule
";
        let settings = FormatSettings {
            spaces_in_parentheses: true,
            spaces_in_square_brackets: true,
            ..FormatSettings::default()
        };

        let actual = Formatter::new(settings).format(input);
        assert!(actual.contains("reg[ addr ] <= data;"));
        assert!(actual.contains("if ( load==1 )"));
    }

    #[test]
    fn formatter_splits_module_header_ports() {
        let input = "module foo(input clk,input rst_n);\nendmodule\n";
        let actual = Formatter::new(FormatSettings::default()).format(input);

        assert_eq!(
            actual,
            "module foo(input clk,\n          input rst_n);\nendmodule\n"
        );
    }

    #[test]
    fn formatter_exits_single_line_blocks_before_following_statements() {
        let input = "\
module foo;
always @(posedge clk)
q<=d;
assign out=flag;
endmodule
";

        let expected = "\
module foo;
    always @(posedge clk)
        q <= d;
    assign out = flag;
endmodule
";

        let actual = Formatter::new(FormatSettings::default()).format(input);
        assert_eq!(actual, expected);
    }
}
