use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;

use super::util::{leading_spaces, spaces, split_line_comment};

static SPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
static HASH_PAREN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"#\s*\(").unwrap());
static OPEN_PAREN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(\s*").unwrap());
static CLOSE_PAREN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*\)").unwrap());
static COMMA_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*,\s*").unwrap());
static CLOSE_SEMICOLON_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\)\s*;").unwrap());

pub fn apply(buffer: &mut Vec<String>) {
    let mut index = 0usize;

    while index < buffer.len() {
        if !buffer[index].trim_start().starts_with("module ") {
            index += 1;
            continue;
        }

        let Some(end_index) = find_module_header_end(buffer, index) else {
            break;
        };

        let replacement = align_module_header(&buffer[index..=end_index]);
        let replacement_len = replacement.len();
        buffer.splice(index..=end_index, replacement);
        index += replacement_len;
    }
}

fn find_module_header_end(buffer: &[String], start_index: usize) -> Option<usize> {
    for (offset, line) in buffer[start_index..].iter().enumerate() {
        let (code, _) = split_line_comment(line);
        let normalized = code.trim_end();
        if normalized.ends_with(");") || normalized.ends_with(';') {
            return Some(start_index + offset);
        }
    }
    None
}

fn align_module_header(lines: &[String]) -> Vec<String> {
    let mut comments: HashMap<String, Vec<String>> = HashMap::new();
    let start_indent = leading_spaces(&lines[0]);
    let clean_lines = remove_comments(lines, &mut comments);

    let joined = clean_lines
        .into_iter()
        .map(|line| order_line(&line))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if joined.is_empty() {
        return lines.to_vec();
    }

    let mut aligned = bas_align(&joined);
    if aligned.is_empty() {
        return vec![indent(start_indent, &joined)];
    }

    for line in &mut aligned {
        *line = indent(start_indent, line);
    }

    let max_width = aligned.iter().map(|line| line.len()).max().unwrap_or(0);
    let mut result = Vec::new();

    for line in aligned {
        let last_token = line.split_whitespace().last().unwrap_or_default();
        let comment_key = if comments.contains_key(last_token) {
            Some(last_token.to_string())
        } else {
            let trimmed = last_token.trim_end_matches(");");
            comments.contains_key(trimmed).then(|| trimmed.to_string())
        };

        if let Some(comment_key) = comment_key {
            if let Some(comment_lines) = comments.get(&comment_key) {
                result.extend(align_comment_block(&line, max_width, comment_lines));
                continue;
            }
        }

        result.push(line);
    }

    result
}

fn remove_comments(lines: &[String], comments: &mut HashMap<String, Vec<String>>) -> Vec<String> {
    let mut cleaned = Vec::with_capacity(lines.len());
    let mut block_comment_key: Option<String> = None;
    let mut block_comment_lines: Vec<String> = Vec::new();

    for line in lines {
        if block_comment_key.is_some() {
            block_comment_lines.push(line.trim().to_string());
            cleaned.push(String::new());

            if line.contains("*/") {
                if let Some(key) = block_comment_key.take() {
                    comments.insert(key, std::mem::take(&mut block_comment_lines));
                }
            }
            continue;
        }

        if let Some(index) = line.find("/*") {
            let prefix = line[..index].trim_end();
            let key = prefix
                .split_whitespace()
                .last()
                .map(str::to_string)
                .unwrap_or_default();

            if line[index..].contains("*/") {
                comments.insert(key, vec![line[index..].trim().to_string()]);
            } else {
                block_comment_key = Some(key);
                block_comment_lines.push(line[index..].trim().to_string());
            }

            cleaned.push(prefix.to_string());
            continue;
        }

        if let Some(index) = line.find("//") {
            let prefix = line[..index].trim_end();
            let key = prefix
                .split_whitespace()
                .last()
                .map(str::to_string)
                .unwrap_or_default();
            comments.insert(key, vec![line[index..].trim().to_string()]);
            cleaned.push(prefix.to_string());
            continue;
        }

        cleaned.push(line.to_string());
    }

    cleaned
}

fn order_line(line: &str) -> String {
    let collapsed = HASH_PAREN_RE.replace_all(line.trim(), "#(").into_owned();
    let collapsed = SPACE_RE.replace_all(&collapsed, " ").into_owned();
    let collapsed = OPEN_PAREN_RE.replace_all(&collapsed, "(").into_owned();
    let collapsed = CLOSE_PAREN_RE.replace_all(&collapsed, ")").into_owned();
    let collapsed = CLOSE_SEMICOLON_RE
        .replace_all(&collapsed, ");")
        .into_owned();
    COMMA_RE.replace_all(&collapsed, ", ").trim().to_string()
}

fn bas_align(module_line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let has_parameters = module_line.contains("#(");
    let mut indent_size = 0usize;
    let mut search_from = 0usize;
    let mut end_param_line = 0usize;

    if has_parameters {
        let Some(param_open) = module_line.find("#(") else {
            return vec![module_line.to_string()];
        };
        let Some(param_close_relative) = module_line[param_open + 2..].find(')') else {
            return vec![module_line.to_string()];
        };
        let param_close = param_open + 2 + param_close_relative;

        result.push(module_line[..param_open + 2].to_string());

        let parameters = split_csv(&module_line[param_open + 2..param_close]);
        if parameters.len() <= 1 {
            result[0].push_str(&module_line[param_open + 2..=param_close]);
        } else {
            result[0].push_str(&format!("{},", parameters[0]));
            for parameter in parameters
                .iter()
                .skip(1)
                .take(parameters.len().saturating_sub(2))
            {
                result.push(indent(param_open + 1, &format!("{parameter},")));
            }
            if let Some(last_parameter) = parameters.last() {
                result.push(format!(
                    "{}{}",
                    indent(param_open + 1, last_parameter),
                    module_line
                        .as_bytes()
                        .get(param_close)
                        .map(|character| *character as char)
                        .unwrap_or(')')
                ));
            }
        }

        indent_size = param_open;
        search_from = param_close;
    }

    let Some(port_open_relative) = module_line[search_from..].find('(') else {
        return vec![module_line.to_string()];
    };
    let port_open = search_from + port_open_relative;
    let Some(port_close) = module_line.rfind(')') else {
        return vec![module_line.to_string()];
    };

    if has_parameters {
        result.push(indent(
            indent_size,
            &module_line[search_from + 1..=port_open],
        ));
        end_param_line = result.len().saturating_sub(1);
        indent_size += 1;
    } else {
        result.push(module_line[..=port_open].to_string());
        indent_size = port_open;
    }

    let ports = split_csv(&module_line[port_open + 1..port_close]);
    if ports.is_empty() {
        result[end_param_line].push_str(&module_line[port_close..]);
        return result;
    }

    if ports.len() == 1 {
        result[end_param_line].push_str(&ports[0]);
        result[end_param_line].push_str(&module_line[port_close..]);
        return result;
    }

    result[end_param_line].push_str(&format!("{},", ports[0]));
    for port in ports.iter().skip(1).take(ports.len().saturating_sub(2)) {
        result.push(indent(indent_size, &format!("{port},")));
    }
    if let Some(last_port) = ports.last() {
        result.push(format!(
            "{}{}",
            indent(indent_size, last_port),
            &module_line[port_close..]
        ));
    }

    result
}

fn split_csv(values: &str) -> Vec<String> {
    values
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn align_comment_block(line: &str, width: usize, comment_lines: &[String]) -> Vec<String> {
    if comment_lines.is_empty() {
        return vec![line.to_string()];
    }

    let padding = spaces(width.saturating_sub(line.len()) + 1);
    let blank_prefix = format!("{}{}", " ".repeat(line.len()), padding);
    let mut result = Vec::with_capacity(comment_lines.len());

    for (index, comment_line) in comment_lines.iter().enumerate() {
        if index == 0 {
            result.push(format!("{line}{padding}{comment_line}"));
        } else {
            result.push(format!("{blank_prefix}{comment_line}"));
        }
    }

    result
}

fn indent(size: usize, line: &str) -> String {
    format!("{}{}", spaces(size), line)
}
