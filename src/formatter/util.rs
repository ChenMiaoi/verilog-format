pub fn spaces(count: usize) -> String {
    " ".repeat(count)
}

pub fn leading_spaces(line: &str) -> usize {
    line.chars()
        .take_while(|character| *character == ' ')
        .count()
}

pub fn split_line_comment(line: &str) -> (&str, Option<&str>) {
    match line.find("//") {
        Some(index) => (&line[..index], Some(&line[index..])),
        None => (line, None),
    }
}

pub fn split_inline_comment(line: &str) -> (&str, Option<&str>) {
    let line_comment = line.find("//");
    let block_comment = line.find("/*");

    let index = match (line_comment, block_comment) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(index), None) | (None, Some(index)) => Some(index),
        (None, None) => None,
    };

    match index {
        Some(index) => (&line[..index], Some(&line[index..])),
        None => (line, None),
    }
}

pub fn code_for_match(line: &str) -> &str {
    split_inline_comment(line).0.trim_end()
}

pub fn is_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
        || trimmed.starts_with("*/")
}
