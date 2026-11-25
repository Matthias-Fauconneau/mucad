// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The wrapping of line comments into the maximal width.
//!
//! A comment behind code continues at its column on the next lines, e.g.:
//!
//! ```mu
//! x = 1; // A comment behind code, which is too long for one line,
//!        // continues at its column.
//! ```
//!
//! The texts of other comments are reflowed as markdown, e.g. of paragraphs and list items, if
//! a line of a text is too long. Other blocks are kept as they are, e.g. code blocks, headings,
//! tables and block quotes.

use std::ops::Range;

use crate::layout::Line;

/// The prefixes of line comments, longest first.
const PREFIXES: [&str; 3] = ["///", "//!", "//"];

/// Consecutive lines, which are wrapped together: a line with a comment behind code and the
/// line comments, which continue it, or line comments with the same prefix and indentation, or
/// another line.
pub struct CommentBlock {
    lines: Vec<Line>,
}

impl CommentBlock {
    /// Split lines into blocks.
    pub fn split(lines: Vec<Line>) -> Vec<Self> {
        let mut blocks: Vec<Self> = Vec::new();
        for line in lines {
            match blocks.last_mut() {
                Some(block) if block.continues(&line) => block.lines.push(line),
                _ => blocks.push(Self { lines: vec![line] }),
            }
        }
        blocks
    }

    /// Whether a line continues the block.
    fn continues(&self, line: &Line) -> bool {
        let first = &self.lines[0];
        match self.prefix() {
            _ if first.trailing_line_comment().is_some() => line.continues_comment(),
            Some(prefix) => {
                line.indent() == first.indent()
                    && line
                        .full_comment()
                        .is_some_and(|text| split_comment(text).0 == prefix)
            }
            None => false,
        }
    }

    /// The prefix of the line comments, e.g. `///`, unless the block starts with code.
    fn prefix(&self) -> Option<&'static str> {
        self.lines[0]
            .full_comment()
            .map(|text| split_comment(text).0)
    }

    /// The lines with the comment behind code on one line, which joins the line comments
    /// continuing it, so that the code is aligned like without them.
    pub fn joined(self) -> Vec<Line> {
        let mut lines = self.lines.into_iter();
        let Some(mut first) = lines.next() else {
            return Vec::new();
        };
        let Some((_, text)) = first.trailing_line_comment() else {
            return std::iter::once(first).chain(lines).collect();
        };

        let mut text = text.to_string();
        for line in lines {
            let content = split_comment(line.full_comment().unwrap_or_default()).1;
            text = format!("{text} {content}");
        }
        first.set_trailing_line_comment(&text);
        vec![first]
    }

    /// The lines with their comments wrapped into the maximal width.
    pub fn wrap(self, max_width: usize) -> Vec<Line> {
        match self.prefix() {
            Some(prefix) => self.reflow(prefix, max_width),
            None => self
                .joined()
                .into_iter()
                .flat_map(|line| wrap_trailing_comment(line, max_width))
                .collect(),
        }
    }

    /// Reflow the texts of the line comments, which have a line, which is too long.
    ///
    /// The first line of a text keeps what is in front of it, e.g. the marker of a list item,
    /// the other lines are indented by its width.
    fn reflow(self, prefix: &str, max_width: usize) -> Vec<Line> {
        let indent = self.lines[0].indent();
        let contents: Vec<&str> = self
            .lines
            .iter()
            .map(|line| split_comment(line.full_comment().unwrap_or_default()).1)
            .collect();
        let markdown = contents.join("\n");
        // The offsets of the lines in the markdown.
        let starts: Vec<usize> = contents
            .iter()
            .scan(0, |start, content| {
                let line = *start;
                *start += content.len() + 1;
                Some(line)
            })
            .collect();
        let line_of = |offset: usize| starts.partition_point(|&start| start <= offset) - 1;
        // The width of the text behind `/// `.
        let width = max_width.saturating_sub(indent + prefix.len() + 1);

        let mut reflowed: Vec<Option<Vec<Line>>> = self
            .lines
            .iter()
            .map(|line| Some(vec![line.clone()]))
            .collect();
        for text in texts(&markdown) {
            let (first, last) = (line_of(text.start), line_of(text.end.saturating_sub(1)));
            if self.lines[first..=last]
                .iter()
                .all(|line| line.width() <= max_width)
            {
                continue;
            }

            let head = &markdown[starts[first]..text.start];
            let head_width = head.chars().count();
            let words = markdown[text].split_whitespace();
            let wrapped = wrap_words(words, width.saturating_sub(head_width));
            let wrapped = wrapped.iter().enumerate().map(|(i, words)| {
                let head = match i {
                    0 => head.to_string(),
                    _ => " ".repeat(head_width),
                };
                comment_line(indent, prefix, &format!("{head}{words}"))
            });
            reflowed[first] = Some(wrapped.collect());
            reflowed[first + 1..=last].fill(None);
        }
        reflowed.into_iter().flatten().flatten().collect()
    }
}

/// A line with a comment behind code, which continues at its column on the next lines, if the
/// line is too long.
fn wrap_trailing_comment(mut line: Line, max_width: usize) -> Vec<Line> {
    let Some((column, text)) = line
        .trailing_line_comment()
        .filter(|_| line.width() > max_width)
    else {
        return vec![line];
    };
    // The text behind `// `.
    let width = max_width.saturating_sub(column + 3);
    let mut contents = wrap_words(split_comment(text).1.split_whitespace(), width).into_iter();
    line.set_trailing_line_comment(&comment("//", &contents.next().unwrap_or_default()));
    std::iter::once(line)
        .chain(contents.map(|content| comment_line(column, "//", &content)))
        .collect()
}

/// The prefix of a line comment, e.g. `///`, and the text behind it and a space.
fn split_comment(text: &str) -> (&'static str, &str) {
    let prefix = PREFIXES
        .into_iter()
        .find(|prefix| text.starts_with(prefix))
        .unwrap_or("//");
    let content = &text[prefix.len()..];
    (prefix, content.strip_prefix(' ').unwrap_or(content))
}

/// A line comment, e.g. `/// text`, or only its prefix without a text.
fn comment(prefix: &str, content: &str) -> String {
    match content.is_empty() {
        true => prefix.to_string(),
        false => format!("{prefix} {content}"),
    }
}

/// A line of a line comment at an indentation.
fn comment_line(indent: usize, prefix: &str, content: &str) -> Line {
    let mut line = Line::new(indent);
    line.push_str(&comment(prefix, content));
    line
}

/// Words wrapped into lines of a width, each line with at least one word.
fn wrap_words<'a>(words: impl IntoIterator<Item = &'a str>, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in words {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

/// The ranges of the texts in markdown, which can be wrapped, e.g. of a paragraph or of a list
/// item, but not of a heading, a table, a block quote or a code block.
///
/// A text ends at a hard line break, which is kept.
fn texts(markdown: &str) -> Vec<Range<usize>> {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let inline = |event: &Event| match event {
        Event::Text(_)
        | Event::Code(_)
        | Event::InlineMath(_)
        | Event::InlineHtml(_)
        | Event::FootnoteReference(_)
        | Event::SoftBreak => true,
        Event::Start(tag) => matches!(
            tag,
            Tag::Emphasis
                | Tag::Strong
                | Tag::Strikethrough
                | Tag::Superscript
                | Tag::Subscript
                | Tag::Link { .. }
                | Tag::Image { .. }
        ),
        Event::End(tag) => matches!(
            tag,
            TagEnd::Emphasis
                | TagEnd::Strong
                | TagEnd::Strikethrough
                | TagEnd::Superscript
                | TagEnd::Subscript
                | TagEnd::Link
                | TagEnd::Image
        ),
        _ => false,
    };
    let kept = |tag: &TagEnd| {
        matches!(
            tag,
            TagEnd::Heading(_)
                | TagEnd::BlockQuote(_)
                | TagEnd::CodeBlock
                | TagEnd::HtmlBlock
                | TagEnd::Table
        )
    };

    let mut texts = Vec::new();
    let mut text: Option<Range<usize>> = None;
    // The number of blocks, whose text is kept.
    let mut depth = 0usize;
    for (event, range) in Parser::new_ext(markdown, Options::ENABLE_TABLES).into_offset_iter() {
        if inline(&event) {
            if depth == 0 {
                let start = text.take().map_or(range.start, |text| text.start);
                text = Some(start..range.end);
            }
            continue;
        }
        texts.extend(text.take());
        match &event {
            Event::Start(tag) if kept(&tag.to_end()) => depth += 1,
            Event::End(tag) if kept(tag) => depth -= 1,
            _ => {}
        }
    }
    texts.extend(text);
    texts
}
