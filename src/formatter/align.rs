use super::util::{spaces, split_line_comment};

pub fn align_blocking_assignments(buffer: &mut [String]) {
    align_consecutive(buffer, find_blocking_assignment_index);
}

pub fn align_no_blocking_assignments(buffer: &mut [String]) {
    align_consecutive(buffer, find_nonblocking_assignment_index);
}

pub fn align_line_comments(buffer: &mut [String]) {
    align_consecutive(buffer, find_line_comment_index);
}

fn align_consecutive(buffer: &mut [String], matcher: fn(&str) -> Option<usize>) {
    let mut blocks: Vec<(Vec<usize>, usize)> = Vec::new();
    let mut current_indices: Vec<usize> = Vec::new();
    let mut current_align = 0usize;
    let mut in_block_comment = false;

    for (line_index, line) in buffer.iter().enumerate() {
        let trimmed = line.trim_start();

        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            flush_block(&mut blocks, &mut current_indices, current_align);
            current_align = 0;
            continue;
        }

        if trimmed.starts_with("/*") && !trimmed.contains("*/") {
            in_block_comment = true;
            flush_block(&mut blocks, &mut current_indices, current_align);
            current_align = 0;
            continue;
        }

        match matcher(line) {
            Some(index) => {
                current_align = current_align.max(index);
                current_indices.push(line_index);
            }
            None => {
                flush_block(&mut blocks, &mut current_indices, current_align);
                current_align = 0;
            }
        }
    }

    flush_block(&mut blocks, &mut current_indices, current_align);

    for (indices, align_index) in blocks {
        for index in indices {
            let line = &buffer[index];
            if let Some(key_index) = matcher(line) {
                if key_index < align_index {
                    let (left, right) = line.split_at(key_index);
                    buffer[index] = format!("{left}{}{right}", spaces(align_index - key_index));
                }
            }
        }
    }
}

fn flush_block(
    blocks: &mut Vec<(Vec<usize>, usize)>,
    current_indices: &mut Vec<usize>,
    align_index: usize,
) {
    if current_indices.len() >= 2 {
        blocks.push((std::mem::take(current_indices), align_index));
    } else {
        current_indices.clear();
    }
}

fn find_line_comment_index(line: &str) -> Option<usize> {
    line.find("//")
}

fn find_nonblocking_assignment_index(line: &str) -> Option<usize> {
    let (code, _) = split_line_comment(line);
    if !code.contains(';') {
        return None;
    }
    code.find("<=")
}

fn find_blocking_assignment_index(line: &str) -> Option<usize> {
    let (code, _) = split_line_comment(line);
    if !code.contains(';') {
        return None;
    }

    let bytes = code.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'=' && is_blocking_assignment_operator(bytes, index) {
            return Some(index);
        }
    }
    None
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
