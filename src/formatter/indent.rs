use once_cell::sync::Lazy;
use regex::Regex;

use crate::config::FormatSettings;

use super::util::{code_for_match, is_comment_line};

static FOR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^for.*\)$").unwrap());
static FOR_BEGIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^for\s*\(.*\)\s*begin$").unwrap());
static WHILE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^while.*\)$").unwrap());
static WHILE_BEGIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^while\s*\(.*\)\s*begin$").unwrap());
static REPEAT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^repeat.*\)$").unwrap());
static REPEAT_BEGIN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^repeat\s*\(.*\)\s*begin$").unwrap());
static FOREVER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^forever$").unwrap());
static FOREVER_BEGIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^forever\s*begin$").unwrap());
static INITIAL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^initial$").unwrap());
static INITIAL_BEGIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^initial\s*begin$").unwrap());
static ALWAYS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^always.*\)$").unwrap());
static ALWAYS_BEGIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^always.*\bbegin$").unwrap());
static CASE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(case|casex|casez)\b.*\)$").unwrap());
static BEGIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^begin$").unwrap());
static END_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^end$").unwrap());
static FUNCTION_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^function\b.*$").unwrap());
static TASK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^task\b.+$").unwrap());
static ENDFUNCTION_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^endfunction$").unwrap());
static ENDTASK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^endtask$").unwrap());
static MODULE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^module\b.*$").unwrap());
static ENDMODULE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^endmodule$").unwrap());
static IF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^if\b.*\)$").unwrap());
static IF_BEGIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^if\b.*\bbegin$").unwrap());
static ELSE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^else$").unwrap());
static ELSE_BEGIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^else\s*begin$").unwrap());
static ELSE_IF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^else\s*if\b.*\)$").unwrap());
static ELSE_IF_BEGIN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^else\s*if\b.*\bbegin$").unwrap());

pub fn apply(buffer: &mut [String], settings: &FormatSettings) {
    let mut context = Context::new(settings);

    for line in buffer.iter_mut() {
        let trimmed = line.trim().to_string();
        *line = process_line(&mut context, &trimmed);
    }
}

fn process_line(context: &mut Context<'_>, line: &str) -> String {
    let passes = if context.states.len() <= 1 {
        1
    } else {
        context.states.len()
    };

    for _ in 0..passes {
        if let Some(indented) = handle_block_comment(context, line) {
            context.finalize_line_states();
            return indented;
        }
        if let Some(indented) = handle_simple_block(context, line, SimpleKind::For) {
            context.finalize_line_states();
            return indented;
        }
        if let Some(indented) = handle_simple_block(context, line, SimpleKind::While) {
            context.finalize_line_states();
            return indented;
        }
        if let Some(indented) = handle_simple_block(context, line, SimpleKind::Repeat) {
            context.finalize_line_states();
            return indented;
        }
        if let Some(indented) = handle_simple_block(context, line, SimpleKind::Forever) {
            context.finalize_line_states();
            return indented;
        }
        if let Some(indented) = handle_case(context, line) {
            context.finalize_line_states();
            return indented;
        }
        if let Some(indented) = handle_if(context, line) {
            context.finalize_line_states();
            return indented;
        }
        if let Some(indented) = handle_callable(context, line, CallableKind::Task) {
            context.finalize_line_states();
            return indented;
        }
        if let Some(indented) = handle_callable(context, line, CallableKind::Function) {
            context.finalize_line_states();
            return indented;
        }
        if let Some(indented) = handle_simple_block(context, line, SimpleKind::Initial) {
            context.finalize_line_states();
            return indented;
        }
        if let Some(indented) = handle_simple_block(context, line, SimpleKind::Always) {
            context.finalize_line_states();
            return indented;
        }
        if let Some(indented) = handle_module(context, line) {
            context.finalize_line_states();
            return indented;
        }
    }

    let indented = if line.is_empty() {
        String::new()
    } else {
        context.indent(line)
    };
    context.finalize_line_states();
    indented
}

fn handle_block_comment(context: &mut Context<'_>, line: &str) -> Option<String> {
    if line.starts_with("/*") {
        context.states.push(State::BlockComment(BlockCommentState {
            stage: BlockCommentStage::Init,
        }));
    }

    let state = context.states.pop()?;

    let State::BlockComment(mut state) = state else {
        context.states.push(state);
        return None;
    };

    match state.stage {
        BlockCommentStage::Init => {
            if line.starts_with("/*") && line.contains("*/") {
                None
            } else if line.starts_with("/*") {
                state.stage = BlockCommentStage::Comment;
                context.states.push(State::BlockComment(state));
                Some(context.indent(line))
            } else {
                context.states.push(State::BlockComment(state));
                None
            }
        }
        BlockCommentStage::Comment => {
            if !line.contains("*/") {
                context.states.push(State::BlockComment(state));
            }
            Some(context.indent(&format!(" {line}")))
        }
    }
}

fn handle_simple_block(context: &mut Context<'_>, line: &str, kind: SimpleKind) -> Option<String> {
    let code = code_for_match(line);

    if matches_simple_start(kind, code) || matches_simple_start_with_begin(kind, code) {
        context.states.push(State::Simple(SimpleBlockState {
            kind,
            base_indent: context.count_indent,
            stage: SimpleStage::Init,
            block_state: BlockState::Init,
        }));
    }

    let state = context.states.pop()?;

    let State::Simple(mut state) = state else {
        context.states.push(state);
        return None;
    };

    if state.kind != kind {
        context.states.push(State::Simple(state));
        return None;
    }

    match state.stage {
        SimpleStage::Init => {
            if matches_simple_start(kind, code) {
                context.mark_parent_body_started();
                context.add_count_indent();
                state.stage = SimpleStage::Active;
                state.block_state = BlockState::MaybeBlock;
                let indented = context.indent_with(state.base_indent, line);
                context.states.push(State::Simple(state));
                Some(indented)
            } else if matches_simple_start_with_begin(kind, code) {
                context.mark_parent_body_started();
                context.add_count_indent();
                state.stage = SimpleStage::Active;
                state.block_state = BlockState::InBlock;
                let indented = context.indent_with(state.base_indent, line);
                context.states.push(State::Simple(state));
                Some(indented)
            } else {
                context.states.push(State::Simple(state));
                None
            }
        }
        SimpleStage::Active => match state.block_state {
            BlockState::Init => {
                context.states.push(State::Simple(state));
                None
            }
            BlockState::MaybeBlock => {
                if BEGIN_RE.is_match(code) {
                    state.block_state = BlockState::InBlock;
                    let indented = context.indent_with(state.base_indent, line);
                    context.states.push(State::Simple(state));
                    Some(indented)
                } else if line.is_empty() {
                    context.res_count_indent();
                    Some(context.indent_with(state.base_indent, line))
                } else {
                    state.block_state = BlockState::PendingNoBlock;
                    context.states.push(State::Simple(state));
                    None
                }
            }
            BlockState::InBlock => {
                if END_RE.is_match(code) {
                    context.res_count_indent();
                    Some(context.indent_with(state.base_indent, line))
                } else {
                    context.states.push(State::Simple(state));
                    None
                }
            }
            BlockState::NoBlock => {
                context.res_count_indent();
                None
            }
            BlockState::PendingNoBlock => {
                context.states.push(State::Simple(state));
                None
            }
        },
    }
}

fn handle_case(context: &mut Context<'_>, line: &str) -> Option<String> {
    let code = code_for_match(line);

    if CASE_RE.is_match(code) {
        context.states.push(State::Case(CaseState {
            base_indent: context.count_indent,
            stage: CaseStage::Init,
        }));
    }

    let state = context.states.pop()?;

    let State::Case(mut state) = state else {
        context.states.push(state);
        return None;
    };

    match state.stage {
        CaseStage::Init => {
            if CASE_RE.is_match(code) {
                context.mark_parent_body_started();
                context.add_count_indent();
                state.stage = CaseStage::Case;
                let indented = context.indent_with(state.base_indent, line);
                context.states.push(State::Case(state));
                Some(indented)
            } else {
                context.states.push(State::Case(state));
                None
            }
        }
        CaseStage::Case => {
            if BEGIN_RE.is_match(code) {
                let current_indent = context.count_indent;
                context.add_count_indent();
                context.states.push(State::Case(state));
                Some(context.indent_with(current_indent, line))
            } else if END_RE.is_match(code) {
                context.res_count_indent();
                context.states.push(State::Case(state));
                Some(context.indent(line))
            } else if matches_endcase(code) {
                context.count_indent = state.base_indent;
                Some(context.indent_with(state.base_indent, line))
            } else {
                context.states.push(State::Case(state));
                None
            }
        }
    }
}

fn handle_callable(context: &mut Context<'_>, line: &str, kind: CallableKind) -> Option<String> {
    let code = code_for_match(line);

    if matches_callable_start(kind, code) {
        context.states.push(State::Callable(CallableState {
            kind,
            base_indent: context.count_indent,
            stage: CallableStage::Init,
        }));
    }

    let state = context.states.pop()?;

    let State::Callable(mut state) = state else {
        context.states.push(state);
        return None;
    };

    if state.kind != kind {
        context.states.push(State::Callable(state));
        return None;
    }

    match state.stage {
        CallableStage::Init => {
            if matches_callable_start(kind, code) {
                context.mark_parent_body_started();
                context.add_count_indent();
                state.stage = CallableStage::Body;
                let indented = context.indent_with(state.base_indent, line);
                context.states.push(State::Callable(state));
                Some(indented)
            } else {
                context.states.push(State::Callable(state));
                None
            }
        }
        CallableStage::Body => {
            if matches_begin_anywhere(code) {
                let current_indent = context.count_indent;
                context.add_count_indent();
                context.states.push(State::Callable(state));
                Some(context.indent_with(current_indent, line))
            } else if END_RE.is_match(code) {
                context.res_count_indent();
                context.states.push(State::Callable(state));
                Some(context.indent(line))
            } else if matches_callable_end(kind, code) {
                context.count_indent = state.base_indent;
                Some(context.indent_with(state.base_indent, line))
            } else {
                context.states.push(State::Callable(state));
                None
            }
        }
    }
}

fn handle_if(context: &mut Context<'_>, line: &str) -> Option<String> {
    let code = code_for_match(line);

    if IF_RE.is_match(code) || IF_BEGIN_RE.is_match(code) {
        context.states.push(State::If(IfState {
            base_indent: context.count_indent,
            stage: IfStage::Init,
            block_state: BlockState::Init,
        }));
    }

    let state = context.states.pop()?;

    let State::If(mut state) = state else {
        context.states.push(state);
        return None;
    };

    let mut keep_state = true;

    let response = match state.stage {
        IfStage::Init => {
            if IF_RE.is_match(code) {
                context.mark_parent_body_started();
                context.add_count_indent();
                state.stage = IfStage::If;
                state.block_state = BlockState::MaybeBlock;
                Some(context.indent_with(state.base_indent, line))
            } else if IF_BEGIN_RE.is_match(code) {
                context.mark_parent_body_started();
                context.add_count_indent();
                state.stage = IfStage::If;
                state.block_state = BlockState::InBlock;
                Some(context.indent_with(state.base_indent, line))
            } else {
                None
            }
        }
        IfStage::If => match state.block_state {
            BlockState::Init => None,
            BlockState::MaybeBlock => {
                if is_comment_line(line) {
                    Some(context.indent_with(state.base_indent, line))
                } else if BEGIN_RE.is_match(code) {
                    state.block_state = BlockState::InBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else {
                    state.block_state = BlockState::PendingNoBlock;
                    None
                }
            }
            BlockState::InBlock => {
                if END_RE.is_match(code) {
                    state.stage = IfStage::ElseIf;
                    state.block_state = BlockState::Init;
                    Some(context.indent_with(state.base_indent, line))
                } else {
                    None
                }
            }
            BlockState::NoBlock => {
                if ELSE_IF_RE.is_match(code) {
                    state.stage = IfStage::ElseIf;
                    state.block_state = BlockState::MaybeBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else if ELSE_IF_BEGIN_RE.is_match(code) {
                    state.stage = IfStage::ElseIf;
                    state.block_state = BlockState::InBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else if ELSE_RE.is_match(code) {
                    state.stage = IfStage::Else;
                    state.block_state = BlockState::MaybeBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else if ELSE_BEGIN_RE.is_match(code) {
                    state.stage = IfStage::Else;
                    state.block_state = BlockState::InBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else {
                    context.res_count_indent();
                    keep_state = false;
                    None
                }
            }
            BlockState::PendingNoBlock => {
                keep_state = true;
                None
            }
        },
        IfStage::ElseIf => match state.block_state {
            BlockState::Init => {
                if ELSE_IF_RE.is_match(code) {
                    state.block_state = BlockState::MaybeBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else if ELSE_IF_BEGIN_RE.is_match(code) {
                    state.block_state = BlockState::InBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else if ELSE_RE.is_match(code) {
                    state.stage = IfStage::Else;
                    state.block_state = BlockState::MaybeBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else if ELSE_BEGIN_RE.is_match(code) {
                    state.stage = IfStage::Else;
                    state.block_state = BlockState::InBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else {
                    context.res_count_indent();
                    keep_state = false;
                    None
                }
            }
            BlockState::MaybeBlock => {
                if is_comment_line(line) {
                    Some(context.indent_with(state.base_indent, line))
                } else if BEGIN_RE.is_match(code) {
                    state.block_state = BlockState::InBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else {
                    state.block_state = BlockState::PendingNoBlock;
                    None
                }
            }
            BlockState::InBlock => {
                if END_RE.is_match(code) {
                    state.stage = IfStage::Else;
                    state.block_state = BlockState::Init;
                    Some(context.indent_with(state.base_indent, line))
                } else {
                    None
                }
            }
            BlockState::NoBlock => {
                if ELSE_RE.is_match(code) {
                    state.stage = IfStage::Else;
                    state.block_state = BlockState::MaybeBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else if ELSE_BEGIN_RE.is_match(code) {
                    state.stage = IfStage::Else;
                    state.block_state = BlockState::InBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else if ELSE_IF_RE.is_match(code) {
                    state.block_state = BlockState::MaybeBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else if ELSE_IF_BEGIN_RE.is_match(code) {
                    state.block_state = BlockState::InBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else {
                    context.res_count_indent();
                    keep_state = false;
                    None
                }
            }
            BlockState::PendingNoBlock => None,
        },
        IfStage::Else => match state.block_state {
            BlockState::Init => {
                if ELSE_RE.is_match(code) {
                    state.block_state = BlockState::MaybeBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else if ELSE_BEGIN_RE.is_match(code) {
                    state.block_state = BlockState::InBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else {
                    None
                }
            }
            BlockState::MaybeBlock => {
                if is_comment_line(line) {
                    Some(context.indent_with(state.base_indent, line))
                } else if BEGIN_RE.is_match(code) {
                    state.block_state = BlockState::InBlock;
                    Some(context.indent_with(state.base_indent, line))
                } else if line.is_empty() {
                    context.res_count_indent();
                    keep_state = false;
                    None
                } else {
                    state.block_state = BlockState::PendingNoBlock;
                    None
                }
            }
            BlockState::InBlock => {
                if END_RE.is_match(code) {
                    context.res_count_indent();
                    keep_state = false;
                    Some(context.indent_with(state.base_indent, line))
                } else {
                    None
                }
            }
            BlockState::NoBlock => {
                context.res_count_indent();
                keep_state = false;
                None
            }
            BlockState::PendingNoBlock => None,
        },
    };

    if keep_state {
        context.states.push(State::If(state));
    }

    response
}

fn handle_module(context: &mut Context<'_>, line: &str) -> Option<String> {
    let code = code_for_match(line);

    if MODULE_RE.is_match(code) {
        context.states.push(State::Module(ModuleState {
            base_indent: context.count_indent,
            stage: ModuleStage::Init,
        }));
    }

    let state = context.states.pop()?;

    let State::Module(mut state) = state else {
        context.states.push(state);
        return None;
    };

    match state.stage {
        ModuleStage::Init => {
            context.add_count_indent();
            state.stage = ModuleStage::WaitEndmodule;
            let indented = context.indent_with(state.base_indent, line);
            context.states.push(State::Module(state));
            Some(indented)
        }
        ModuleStage::WaitEndmodule => {
            if ENDMODULE_RE.is_match(code) {
                context.res_count_indent();
                Some(context.indent_with(state.base_indent, line))
            } else {
                context.states.push(State::Module(state));
                None
            }
        }
    }
}

fn matches_simple_start(kind: SimpleKind, code: &str) -> bool {
    match kind {
        SimpleKind::For => FOR_RE.is_match(code),
        SimpleKind::While => WHILE_RE.is_match(code),
        SimpleKind::Repeat => REPEAT_RE.is_match(code),
        SimpleKind::Forever => FOREVER_RE.is_match(code),
        SimpleKind::Initial => INITIAL_RE.is_match(code),
        SimpleKind::Always => ALWAYS_RE.is_match(code),
    }
}

fn matches_simple_start_with_begin(kind: SimpleKind, code: &str) -> bool {
    match kind {
        SimpleKind::For => FOR_BEGIN_RE.is_match(code),
        SimpleKind::While => WHILE_BEGIN_RE.is_match(code),
        SimpleKind::Repeat => REPEAT_BEGIN_RE.is_match(code),
        SimpleKind::Forever => FOREVER_BEGIN_RE.is_match(code),
        SimpleKind::Initial => INITIAL_BEGIN_RE.is_match(code),
        SimpleKind::Always => ALWAYS_BEGIN_RE.is_match(code),
    }
}

fn matches_callable_start(kind: CallableKind, code: &str) -> bool {
    match kind {
        CallableKind::Task => TASK_RE.is_match(code),
        CallableKind::Function => FUNCTION_RE.is_match(code),
    }
}

fn matches_callable_end(kind: CallableKind, code: &str) -> bool {
    match kind {
        CallableKind::Task => ENDTASK_RE.is_match(code),
        CallableKind::Function => ENDFUNCTION_RE.is_match(code),
    }
}

fn matches_begin_anywhere(code: &str) -> bool {
    code.split_whitespace().last() == Some("begin")
}

fn matches_endcase(code: &str) -> bool {
    code == "endcase"
}

#[derive(Debug)]
struct Context<'a> {
    settings: &'a FormatSettings,
    states: Vec<State>,
    count_indent: usize,
}

impl<'a> Context<'a> {
    fn new(settings: &'a FormatSettings) -> Self {
        Self {
            settings,
            states: Vec::new(),
            count_indent: 0,
        }
    }

    fn indent(&self, line: &str) -> String {
        format!("{}{}", self.settings.indent(self.count_indent), line)
    }

    fn indent_with(&self, level: usize, line: &str) -> String {
        format!("{}{}", self.settings.indent(level), line)
    }

    fn add_count_indent(&mut self) {
        self.count_indent += 1;
    }

    fn res_count_indent(&mut self) {
        self.count_indent = self.count_indent.saturating_sub(1);
    }

    fn finalize_line_states(&mut self) {
        for state in &mut self.states {
            match state {
                State::Simple(simple_state)
                    if matches!(simple_state.block_state, BlockState::PendingNoBlock) =>
                {
                    simple_state.block_state = BlockState::NoBlock;
                }
                State::If(if_state)
                    if matches!(if_state.block_state, BlockState::PendingNoBlock) =>
                {
                    if_state.block_state = BlockState::NoBlock;
                }
                _ => {}
            }
        }
    }

    fn mark_parent_body_started(&mut self) {
        for state in self.states.iter_mut().rev() {
            match state {
                State::Simple(simple_state)
                    if matches!(simple_state.block_state, BlockState::MaybeBlock) =>
                {
                    simple_state.block_state = BlockState::NoBlock;
                    break;
                }
                State::If(if_state)
                    if !matches!(if_state.stage, IfStage::Init)
                        && matches!(if_state.block_state, BlockState::MaybeBlock) =>
                {
                    if_state.block_state = BlockState::NoBlock;
                    break;
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
enum State {
    BlockComment(BlockCommentState),
    Simple(SimpleBlockState),
    Case(CaseState),
    Callable(CallableState),
    If(IfState),
    Module(ModuleState),
}

#[derive(Debug)]
struct BlockCommentState {
    stage: BlockCommentStage,
}

#[derive(Debug)]
enum BlockCommentStage {
    Init,
    Comment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SimpleKind {
    For,
    While,
    Repeat,
    Forever,
    Initial,
    Always,
}

#[derive(Debug)]
struct SimpleBlockState {
    kind: SimpleKind,
    base_indent: usize,
    stage: SimpleStage,
    block_state: BlockState,
}

#[derive(Debug)]
enum SimpleStage {
    Init,
    Active,
}

#[derive(Debug)]
struct CaseState {
    base_indent: usize,
    stage: CaseStage,
}

#[derive(Debug)]
enum CaseStage {
    Init,
    Case,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallableKind {
    Task,
    Function,
}

#[derive(Debug)]
struct CallableState {
    kind: CallableKind,
    base_indent: usize,
    stage: CallableStage,
}

#[derive(Debug)]
enum CallableStage {
    Init,
    Body,
}

#[derive(Debug)]
struct IfState {
    base_indent: usize,
    stage: IfStage,
    block_state: BlockState,
}

#[derive(Debug)]
enum IfStage {
    Init,
    If,
    ElseIf,
    Else,
}

#[derive(Debug, Clone, Copy)]
enum BlockState {
    Init,
    MaybeBlock,
    PendingNoBlock,
    InBlock,
    NoBlock,
}

#[derive(Debug)]
struct ModuleState {
    base_indent: usize,
    stage: ModuleStage,
}

#[derive(Debug)]
enum ModuleStage {
    Init,
    WaitEndmodule,
}
