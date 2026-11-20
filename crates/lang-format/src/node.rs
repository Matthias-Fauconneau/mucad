// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The nodes of formatted code, which are built from the AST and laid out in a maximal width.
//!
//! A node only has the structure of the code, e.g. a [`Node::Group`] of the items of a list,
//! which is on one line, or with each item on its own line. Which one, is decided when the nodes
//! are rendered into [`Paragraph`]s of lines, see [`Node::render`].

use crate::{
    FormatConfig,
    layout::{Column, Line, Paragraph},
    node,
};

use mucad_lang_base::{CompactString, ToCompactString};
use mucad_lang_parse::ast;

/// A break between texts.
///
/// [`Break::SoftLine`] and [`Break::Line`] are resolved by their group, then breaks next to each
/// other are the strongest of them, which are ordered from [`Break::Space`] to
/// [`Break::Paragraph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Break {
    /// A space, e.g. between two tokens, or where a line of the source broke.
    ///
    /// It is no space at the start of a line, behind a space, an opening bracket or a `.`, and in
    /// front of a separator, a closing bracket, a `.` or a space, but always a space in front of
    /// a single line comment.
    Space,
    /// Nothing, if its group is on one line, otherwise a line break, e.g. behind `(`.
    SoftLine,
    /// A space, if its group is on one line, otherwise a line break, e.g. behind `,`.
    Line,
    /// A line break, e.g. behind a statement or a single line comment.
    Hard,
    /// A blank line between two paragraphs, e.g. of statements.
    Paragraph,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Node {
    #[default]
    Nil,
    Text(CompactString),
    Break(Break),
    /// The start of a cell at a column of a line, which aligns with the cells of other lines.
    Column(Column),
    /// A node, whose lines are indented by one level more.
    Indent(Box<Node>),
    /// Nodes one after another.
    Concat(Vec<Node>),
    /// Nodes on one line, if they fit into the line and have no line break, otherwise their
    /// [`Break::SoftLine`]s and [`Break::Line`]s break the line.
    Group(Box<Node>),
    /// A node, which is only rendered, if its group breaks, e.g. a trailing `,`.
    IfBroken(Box<Node>),
    /// The pieces of a string between `"`, which is continued with `+` on the next line in front
    /// of a piece, which does not fit, e.g.:
    ///
    /// ```mu
    /// path = "M -9,-15 H 4 C 12,-15 12,-7 "
    ///     + "4,-7 H -9 Z";
    /// ```
    Str(Vec<Node>),
    /// A line comment, e.g. `// …`, which is wrapped into the maximal width, with its column in
    /// the source, if it started a line there.
    Comment(CompactString, Option<usize>),
}

impl Node {
    /// Nodes with a separator between them, e.g. `a::b`.
    pub fn join<I>(nodes: I, separator: impl Into<Node>) -> Node
    where
        I: IntoIterator<Item = Node>,
    {
        let separator: Node = separator.into();
        let mut result = Vec::new();
        for (i, node) in nodes.into_iter().enumerate() {
            if i > 0 {
                result.push(separator.clone());
            }
            result.push(node);
        }
        result.into()
    }

    /// Items with `,` in brackets, with the extras inside the brackets, e.g. `(a, b)`.
    ///
    /// The items are on one line, if they fit, otherwise each item is on its own indented line
    /// with a trailing `,`. A comment behind the opening bracket stays on its line.
    pub fn bracketed(
        (open, close): (&str, &str),
        extras: &ast::ItemExtras,
        items: Vec<Node>,
    ) -> Node {
        // Empty brackets do not break, e.g. `()`, only behind a comment.
        if items.is_empty() {
            return node!(open extras.leading extras.trailing close);
        }
        let trailing_comma = Node::IfBroken(Box::new(','.into()));
        let separator = node!(',' Break::Line Node::Column(Column::Separator));
        Node::group(node!(
            open
            Node::indent(node!(
                extras.leading
                Break::SoftLine
                Node::join(items, separator)
                trailing_comma
                extras.trailing
            ))
            Break::SoftLine
            close
        ))
    }

    pub fn indent(node: impl Into<Node>) -> Self {
        Node::Indent(Box::new(node.into()))
    }

    pub fn comment(text: impl Into<CompactString>, source_column: Option<usize>) -> Self {
        Node::Comment(text.into(), source_column)
    }

    pub fn group(node: impl Into<Node>) -> Self {
        Node::Group(Box::new(node.into()))
    }

    /// The width in characters, if the node is on one line.
    fn flat_width(&self) -> usize {
        match &self {
            Node::Text(text) | Node::Comment(text, _) => text.chars().count(),
            Node::Break(Break::Space | Break::Line) => 1,
            Node::Indent(node) | Node::Group(node) => node.flat_width(),
            Node::Concat(nodes) => nodes.iter().map(Node::flat_width).sum(),
            Node::Str(pieces) => 2 + pieces.iter().map(Node::flat_width).sum::<usize>(),
            Node::Nil
            | Node::Column(_)
            | Node::IfBroken(_)
            | Node::Break(Break::SoftLine | Break::Hard | Break::Paragraph) => 0,
        }
    }

    /// The width in characters up to the first possible line break, and whether there is one.
    fn width_to_break(&self) -> (usize, bool) {
        match &self {
            // A comment is wrapped, so the code in front of it does not need its width.
            Node::Break(Break::SoftLine | Break::Line | Break::Hard | Break::Paragraph)
            | Node::Comment(..) => (0, true),
            Node::Indent(node) | Node::Group(node) => node.width_to_break(),
            // A string can break behind its first piece.
            Node::Str(pieces) if pieces.len() > 1 => (1 + pieces[0].flat_width(), true),
            Node::Concat(nodes) => {
                let mut width = 0;
                for node in nodes {
                    let (node_width, found) = node.width_to_break();
                    width += node_width;
                    if found {
                        return (width, true);
                    }
                }
                (width, false)
            }
            node => (node.flat_width(), false),
        }
    }

    /// Whether the node breaks a line, e.g. with a single line comment.
    pub fn has_line_break(&self) -> bool {
        match &self {
            Node::Text(text) => text.contains('\n'),
            Node::Break(line_break) => *line_break >= Break::Hard,
            Node::Indent(node) | Node::Group(node) | Node::IfBroken(node) => node.has_line_break(),
            Node::Concat(nodes) | Node::Str(nodes) => nodes.iter().any(Node::has_line_break),
            Node::Nil | Node::Column(_) | Node::Comment(..) => false,
        }
    }

    /// Compact nodes: flatten concatenations, remove `Nil` nodes and concatenate adjacent texts.
    pub fn compact(nodes: Vec<Node>) -> Self {
        fn flatten(nodes: Vec<Node>, flat: &mut Vec<Node>) {
            for node in nodes {
                match node {
                    Node::Concat(nodes) => flatten(nodes, flat),
                    Node::Nil => {}
                    node => flat.push(node),
                }
            }
        }

        let mut flat = Vec::with_capacity(nodes.len());
        flatten(nodes, &mut flat);

        let mut compacted: Vec<Node> = Vec::with_capacity(flat.len());
        for node in flat {
            match (compacted.last_mut(), node) {
                (Some(Node::Text(last)), Node::Text(next)) => last.push_str(&next),
                (_, node) => compacted.push(node),
            }
        }

        match compacted.len() {
            0 => Node::Nil,
            1 => compacted.remove(0),
            _ => Node::Concat(compacted),
        }
    }

    /// Render the node into code, whose lines are at most as wide as the maximal width, if
    /// possible.
    pub fn render(&self, f: &FormatConfig) -> String {
        let mut state = RenderState {
            max_width: f.max_width,
            indent_width: f.indent_width,
            ..Default::default()
        };
        self.render_recursive(Mode::Broken, 0, &mut state);

        let mut code = String::new();
        for (i, paragraph) in state.paragraphs.iter_mut().enumerate() {
            paragraph.layout(f.max_width);
            if i > 0 {
                code.push_str("\n\n");
            }
            code.push_str(&paragraph.to_string());
        }
        // The output ends with a line break.
        if matches!(state.pending, Some(Break::Hard | Break::Paragraph)) {
            code.push('\n');
        }
        code
    }
}

impl From<Vec<Node>> for Node {
    fn from(nodes: Vec<Node>) -> Self {
        Node::compact(nodes)
    }
}

impl From<Break> for Node {
    fn from(value: Break) -> Self {
        Node::Break(value)
    }
}

impl From<String> for Node {
    fn from(value: String) -> Self {
        Node::Text(value.to_compact_string())
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Node::Text(value.to_compact_string())
    }
}

impl From<char> for Node {
    fn from(value: char) -> Self {
        Node::Text(value.to_compact_string())
    }
}

impl From<CompactString> for Node {
    fn from(value: CompactString) -> Self {
        Node::Text(value)
    }
}

/// Whether the group of a node is on one line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Flat,
    Broken,
}

/// The state of rendering nodes into paragraphs of lines.
#[derive(Default)]
struct RenderState {
    max_width: usize,
    indent_width: usize,
    indent_level: usize,
    /// The strongest break since the last text, which is written in front of the next text.
    pending: Option<Break>,
    paragraphs: Vec<Paragraph>,
    /// The last text ends with a space, an opening bracket or a `.`, which has no space behind it.
    no_space: bool,
}

impl RenderState {
    /// The current line, which is `None` at the start.
    fn line(&mut self) -> Option<&mut Line> {
        self.paragraphs.last_mut().and_then(|p| p.lines.last_mut())
    }

    /// The column, at which the next text starts.
    fn column(&mut self) -> usize {
        let indent_level = self.indent_level;
        match self.pending {
            Some(Break::Hard | Break::Paragraph) => indent_level,
            _ => self.line().map_or(indent_level, |line| line.width()),
        }
    }

    /// Add a break, which is resolved by the mode of its group.
    fn add_break(&mut self, line_break: Break, mode: Mode) {
        let resolved = match (line_break, mode) {
            (Break::SoftLine, Mode::Flat) => return,
            (Break::Line, Mode::Flat) => Break::Space,
            (Break::SoftLine | Break::Line, Mode::Broken) => Break::Hard,
            (line_break, _) => line_break,
        };
        self.pending = self.pending.max(Some(resolved));
    }

    /// Write a text behind the pending break.
    fn write(&mut self, text: &str) {
        // An empty text does not start a line.
        if text.is_empty() {
            return;
        }
        // No space in front of a separator, a closing bracket, an access or a space.
        let pending = match self.pending.take() {
            Some(Break::Space) if text.starts_with([',', ';', ')', ']', '.', ' ']) => None,
            pending => pending,
        };
        let line = Line::new(self.indent_level);
        match pending {
            // No breaks at the start of the output.
            _ if self.paragraphs.is_empty() => {
                self.paragraphs.push(Paragraph { lines: vec![line] })
            }
            None | Some(Break::SoftLine | Break::Line) => {}
            Some(Break::Space) if self.no_space && !text.starts_with("//") => {}
            Some(Break::Space) => {
                if let Some(line) = self.line() {
                    line.push_str(" ");
                }
            }
            Some(Break::Hard) => {
                if let Some(paragraph) = self.paragraphs.last_mut() {
                    paragraph.lines.push(line);
                }
            }
            Some(Break::Paragraph) => self.paragraphs.push(Paragraph { lines: vec![line] }),
        }
        self.no_space = text.ends_with(['(', '[', '.', ' ']);
        if let Some(line) = self.line() {
            line.push_str(text);
        }
    }

    /// Start a cell of the current line at a column, unless a line break is pending.
    fn mark(&mut self, column: Column) {
        if matches!(self.pending, None | Some(Break::Space))
            && let Some(line) = self.line()
        {
            line.mark(column);
        }
    }
}

impl Node {
    /// Render a node in the mode of its group, followed by text of a width up to the next
    /// possible line break, which must fit into the line, too.
    fn render_recursive(&self, mode: Mode, trailing: usize, state: &mut RenderState) {
        match self {
            Node::Text(text) => state.write(text),
            Node::Comment(text, source_column) => {
                state.write(text);
                if let Some(line) = state.line() {
                    line.mark_comment(*source_column);
                }
            }
            Node::Break(line_break) => state.add_break(*line_break, mode),
            Node::Column(column) => state.mark(*column),
            Node::Concat(nodes) => {
                // The width behind each node up to the next possible line break.
                let mut trailings = vec![trailing; nodes.len()];
                for i in (1..nodes.len()).rev() {
                    let (width, found) = nodes[i].width_to_break();
                    trailings[i - 1] = if found { width } else { width + trailings[i] };
                }
                for (node, trailing) in nodes.iter().zip(trailings) {
                    node.render_recursive(mode, trailing, state);
                }
            }
            Node::Indent(node) => {
                state.indent_level += state.indent_width;
                node.render_recursive(mode, trailing, state);
                state.indent_level -= state.indent_width;
            }
            Node::Group(node) => {
                let fits = |state: &mut RenderState| {
                    state.column() + node.flat_width() + trailing <= state.max_width
                };
                let mode = match mode {
                    Mode::Flat => Mode::Flat,
                    Mode::Broken if !node.has_line_break() && fits(state) => Mode::Flat,
                    Mode::Broken => Mode::Broken,
                };
                node.render_recursive(mode, trailing, state);
            }
            Node::Str(pieces) => {
                state.write("\"");
                let indent_level = state.indent_level;
                for (i, piece) in pieces.iter().enumerate() {
                    // The closing `"` and the text behind the string must fit, too.
                    let behind = if i + 1 == pieces.len() {
                        1 + trailing
                    } else {
                        1
                    };
                    if i > 0 && state.column() + piece.flat_width() + behind > state.max_width {
                        state.write("\"");
                        state.indent_level = indent_level + state.indent_width;
                        state.add_break(Break::Hard, mode);
                        state.write("+ \"");
                    }
                    piece.render_recursive(Mode::Flat, 0, state);
                }
                state.write("\"");
                state.indent_level = indent_level;
            }
            Node::IfBroken(node) => {
                if mode == Mode::Broken {
                    node.render_recursive(mode, trailing, state);
                }
            }
            Node::Nil => {}
        }
    }
}
