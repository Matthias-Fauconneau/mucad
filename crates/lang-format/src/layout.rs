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

    /// The width of the line with its indentation.
    pub fn width(&self) -> usize {
        self.indent
            + self
                .cells
                .iter()
                .map(|cell| cell.text.chars().count())
                .sum::<usize>()
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

/// Which cells of lines align.
#[derive(Debug, Clone, Copy)]
enum Align {
    /// The cells at an index, which start at the same column of lines with the same indentation.
    Cell(usize),
    /// The trailing comments.
    Comment,
}

impl Line {
    /// The index of the aligned cell, and its key, which must be the same in all lines of a
    /// table, unless the line is not aligned.
    fn aligned(&self, align: Align) -> Option<(usize, Option<&Key>)> {
        if self.multiline {
            return None;
        }
        match align {
            Align::Cell(cell) => {
                let key = self.key(cell).filter(|key| key.column != Column::Comment)?;
                Some((cell, Some(key)))
            }
            Align::Comment => Some((self.trailing_comment()?, None)),
        }
    }

    /// The width in front of the aligned cell, with the indentation.
    fn start(&self, align: Align) -> Option<usize> {
        let (cell, _) = self.aligned(align)?;
        Some(self.indent + self.width_before(cell))
    }
}

/// Consecutive lines, whose aligned cells start at the same width, e.g.:
///
/// ```mu
/// foo = Circle(10mm);
/// a   = Rect(10);
/// ```
struct Table<'a> {
    lines: &'a mut [Line],
    align: Align,
}

impl<'a> Table<'a> {
    /// Split lines into tables of at least two lines.
    ///
    /// A table ends at a line, whose aligned cell has another key, or another indentation, or
    /// which needs more padding than [`MAX_PADDING`].
    fn split(mut lines: &'a mut [Line], align: Align) -> Vec<Self> {
        let mut tables = Vec::new();
        while !lines.is_empty() {
            let len = Self::len(lines, align);
            let (table, rest) = std::mem::take(&mut lines).split_at_mut(len);
            if table.len() > 1 {
                tables.push(Self {
                    lines: table,
                    align,
                });
            }
            lines = rest;
        }
        tables
    }

    /// The number of lines of the table at the start of the lines, at least one.
    fn len(lines: &[Line], align: Align) -> usize {
        let first = &lines[0];
        let Some((_, key)) = first.aligned(align) else {
            return 1;
        };
        let (mut min, mut max) = (usize::MAX, 0);
        let len = lines
            .iter()
            .take_while(|line| match (align, line.aligned(align)) {
                (Align::Cell(_), Some((_, line_key))) => {
                    line.indent == first.indent && line_key == key
                }
                (Align::Comment, Some(_)) => true,
                (_, None) => false,
            })
            .filter_map(|line| line.start(align))
            .take_while(|&start| {
                (min, max) = (min.min(start), max.max(start));
                max - min <= MAX_PADDING
            })
            .count();
        len.max(1)
    }

    /// The width, at which the aligned cells start.
    fn width(&self) -> usize {
        self.lines
            .iter()
            .filter_map(|line| line.start(self.align))
            .max()
            .unwrap_or_default()
    }

    /// Pad the cells in front of the aligned cells, so that they start at the width.
    fn pad(&mut self) {
        let width = self.width();
        for line in self.lines.iter_mut() {
            if let (Some(start), Some((cell, _))) =
                (line.start(self.align), line.aligned(self.align))
            {
                line.cells[cell - 1]
                    .text
                    .push_str(&" ".repeat(width - start));
            }
        }
    }
}

/// Lines without blank lines between them.
#[derive(Debug, Default)]
pub struct Paragraph {
    pub lines: Vec<Line>,
}

impl Paragraph {
    /// Align the cells of the lines in tables, column by column, then their trailing comments.
    pub fn align(&mut self) {
        /// Align the cells at an index, then the next cells of each table.
        fn align_cells(lines: &mut [Line], cell: usize) {
            for mut table in Table::split(lines, Align::Cell(cell)) {
                table.pad();
                align_cells(table.lines, cell + 1);
            }
        }

        align_cells(&mut self.lines, 1);
        for mut table in Table::split(&mut self.lines, Align::Comment) {
            table.pad();
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
