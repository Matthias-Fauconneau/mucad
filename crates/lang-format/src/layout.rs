// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The layout of formatted code: paragraphs of lines, whose cells are aligned in columns, e.g.:
//!
//! ```mu
//! foo = Circle(10mm);
//! a   = Rect(10);
//! ```

/// The most spaces, which align a column.
///
/// A column, which needs more, is not aligned, so that one long line does not spread a table.
const MAX_PADDING: usize = 8;

/// The kind of a column, at which a cell of a line starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    /// An assignment ` = `, e.g. of a statement, an argument or a tuple item.
    Assign,
    /// Behind a separator `,` of items.
    Separator,
    /// A comment, e.g. `// …` or `/* … */`.
    Comment,
}

/// Where a cell starts in its line, the cells of lines with the same key are aligned.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Key {
    column: Column,
    /// The number of open brackets in front of the cell.
    depth: usize,
    /// The text of the line in front of the innermost open bracket, e.g. `f` of `f(a, b)`, so
    /// that only the items of similar lists align.
    head: String,
}

/// A part of a line, which starts at a column.
#[derive(Debug)]
struct Cell {
    /// The column, at which the cell starts, which is `None` for the first cell.
    key: Option<Key>,
    text: String,
}

/// A line of cells.
#[derive(Debug)]
pub struct Line {
    indent: usize,
    cells: Vec<Cell>,
    /// The text without the padding of the alignment.
    text: String,
    /// The positions of the open brackets in the text.
    brackets: Vec<usize>,
    in_string: bool,
    /// The line contains a line break, e.g. of a multi-line comment, and is not aligned.
    multiline: bool,
}

impl Line {
    pub fn new(indent: usize) -> Self {
        Self {
            indent,
            cells: vec![Cell {
                key: None,
                text: String::new(),
            }],
            text: String::new(),
            brackets: Vec::new(),
            in_string: false,
            multiline: false,
        }
    }

    /// Append text to the last cell.
    pub fn push_str(&mut self, text: &str) {
        let mut escaped = false;
        for c in text.chars() {
            match c {
                _ if escaped => escaped = false,
                '\\' if self.in_string => escaped = true,
                '"' => self.in_string = !self.in_string,
                _ if self.in_string => {}
                '(' | '[' | '{' => self.brackets.push(self.text.len()),
                ')' | ']' | '}' => {
                    self.brackets.pop();
                }
                '\n' => self.multiline = true,
                _ => {}
            }
            self.text.push(c);
        }
        if let Some(cell) = self.cells.last_mut() {
            cell.text.push_str(text);
        }
    }

    /// Start a cell at a column, unless the line is empty, e.g. for a comment on its own line.
    pub fn mark(&mut self, column: Column) {
        if self.text.trim().is_empty() {
            return;
        }
        let head = match self.brackets.last() {
            Some(&open) => self.text[..open].to_string(),
            None => String::new(),
        };
        self.cells.push(Cell {
            key: Some(Key {
                column,
                depth: self.brackets.len(),
                head,
            }),
            text: String::new(),
        });
    }

    /// The key of a cell.
    fn key(&self, cell: usize) -> Option<&Key> {
        self.cells.get(cell).and_then(|cell| cell.key.as_ref())
    }

    /// The width of the cells in front of a cell.
    fn width_before(&self, cell: usize) -> usize {
        self.cells[..cell]
            .iter()
            .map(|cell| cell.text.chars().count())
            .sum()
    }

    /// The index of a trailing comment, behind code.
    fn trailing_comment(&self) -> Option<usize> {
        let last = self.cells.len() - 1;
        let cell = &self.cells[last];
        let comment = cell
            .key
            .as_ref()
            .is_some_and(|key| key.column == Column::Comment);
        let trailing =
            cell.text.trim_start().starts_with("//") || cell.text.trim_end().ends_with("*/");
        (last > 0 && comment && trailing).then_some(last)
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text: String = self.cells.iter().map(|cell| cell.text.as_str()).collect();
        if text.is_empty() {
            Ok(())
        } else {
            write!(f, "{}{text}", " ".repeat(self.indent))
        }
    }
}

/// Lines without blank lines between them.
#[derive(Debug, Default)]
pub struct Paragraph {
    pub lines: Vec<Line>,
}

impl Paragraph {
    /// Align the columns of the lines.
    ///
    /// The cells of consecutive lines with the same indentation align, column by column, while
    /// their keys are the same, then their trailing comments align.
    pub fn align(&mut self) {
        let mut start = 0;
        while start < self.lines.len() {
            let indent = self.lines[start].indent;
            let end = start
                + self.lines[start..]
                    .iter()
                    .take_while(|line| line.indent == indent && !line.multiline)
                    .count();
            align_cells(&mut self.lines[start..end], 1);
            start = end.max(start + 1);
        }

        align_comments(&mut self.lines);
    }
}

/// Align a cell of the lines, which have the same keys of the cells in front of it.
fn align_cells(lines: &mut [Line], cell: usize) {
    let mut start = 0;
    while start < lines.len() {
        let key = lines[start]
            .key(cell)
            .filter(|key| key.column != Column::Comment)
            .cloned();
        let len = match &key {
            Some(key) => table_len(&lines[start..], |line| {
                (line.key(cell) == Some(key)).then(|| line.width_before(cell))
            }),
            None => 1,
        };

        let table = &mut lines[start..start + len];
        if len > 1 {
            pad_table(table, |_| Some(cell));
            align_cells(table, cell + 1);
        }
        start += len;
    }
}

/// Align the trailing comments of consecutive lines.
fn align_comments(lines: &mut [Line]) {
    let mut start = 0;
    while start < lines.len() {
        let len = table_len(&lines[start..], |line| {
            let cell = line.trailing_comment().filter(|_| !line.multiline)?;
            Some(line.indent + line.width_before(cell))
        });

        if len > 1 {
            pad_table(&mut lines[start..start + len], Line::trailing_comment);
        }
        start += len.max(1);
    }
}

/// The number of the first lines, whose cells start at widths, which differ by
/// [`MAX_PADDING`] at most.
fn table_len(lines: &[Line], width: impl Fn(&Line) -> Option<usize>) -> usize {
    let (mut min, mut max) = (usize::MAX, 0);
    lines
        .iter()
        .map_while(width)
        .take_while(|&width| {
            (min, max) = (min.min(width), max.max(width));
            max - min <= MAX_PADDING
        })
        .count()
}

/// Pad the lines, so that a cell of each line starts at the same width.
fn pad_table(lines: &mut [Line], cell: impl Fn(&Line) -> Option<usize>) {
    let start = |line: &Line| cell(line).map(|cell| line.indent + line.width_before(cell));
    let width = lines.iter().filter_map(start).max().unwrap_or_default();
    for line in lines {
        if let Some(cell) = cell(line) {
            let padding = width - start(line).unwrap_or(width);
            line.cells[cell - 1].text.push_str(&" ".repeat(padding));
        }
    }
}

impl std::fmt::Display for Paragraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{line}")?;
        }
        Ok(())
    }
}
