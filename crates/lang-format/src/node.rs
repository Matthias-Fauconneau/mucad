// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    FormatConfig,
    layout::{Column, Line, Paragraph},
    node,
};

use mucad_lang_base::{CompactString, ToCompactString};

/// How to perform a line break
pub enum BreakMode {
    NoBreak,
    WithIndent(usize),
}

impl BreakMode {
    /// Determines the break strategy based on the format config
    pub fn from_layout(nodes: &[Node], max_items: usize, f: &FormatConfig) -> Self {
        let width: usize = nodes.iter().map(|node| node.estimate_width()).sum();
        let too_many_items = max_items > 0 && nodes.len() > max_items;
        let too_wide = width > f.max_width;
        let forced_break = nodes.iter().any(Node::has_line_break);

        if too_many_items || too_wide || forced_break {
            Self::WithIndent(f.indent_width)
        } else {
            Self::NoBreak
        }
    }
}

/// A line break, ordered by its strength: breaks next to each other are the strongest of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Break {
    /// A space, e.g. between two tokens, or where a line of the source broke.
    ///
    /// It is no space at the start of a line, behind a space, an opening bracket or a `.`, and in
    /// front of a separator, a closing bracket, a `.` or a space, but always a space in front of
    /// a single line comment.
    Soft,
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
    Indent {
        width: usize,
        node: Box<Node>,
    },
    Group(Vec<Node>),
}

impl Node {
    /// Intersperses a separator between nodes from any iterable source.
    pub fn hlist<I>(nodes: I, separator: impl Into<Node>) -> Node
    where
        I: IntoIterator<Item = Node>,
    {
        let iter = nodes.into_iter();
        let separator: Node = separator.into();
        // Provide a hint to the allocator if possible
        let (lower, _) = iter.size_hint();
        let mut result = Vec::with_capacity(lower.saturating_mul(2));

        let mut first = true;
        for node in iter {
            if !first {
                result.push(separator.clone());
            }
            result.push(node);
            first = false;
        }

        result.into()
    }

    /// Each node with a separator on its own line, indented, if the indent width is not zero.
    pub fn vlist<I>(nodes: I, separator: impl Into<Node>, indent_width: usize) -> Node
    where
        I: IntoIterator<Item = Node>,
    {
        let sep = separator.into();
        let nodes: Node = nodes
            .into_iter()
            .map(|node| node!(node sep.clone() Break::Hard))
            .collect::<Vec<_>>()
            .into();

        match indent_width {
            0 => nodes,
            width => node!(Break::Hard Node::indent(width, nodes)),
        }
    }

    /// A list of items with an separator
    pub fn list<I>(nodes: I, separator: impl Into<Node>, break_mode: BreakMode) -> Node
    where
        I: IntoIterator<Item = Node>,
    {
        let sep = separator.into();
        match break_mode {
            BreakMode::NoBreak => Self::hlist(
                nodes,
                node!(sep Node::Column(Column::Separator) Break::Soft),
            ),
            BreakMode::WithIndent(indent_width) => Self::vlist(nodes, sep, indent_width),
        }
    }

    /// Items with `,` in brackets, with the extras inside the brackets, e.g. `(a, b)`.
    ///
    /// Each item is on its own indented line, if the items are too many or too wide, see
    /// [`BreakMode::from_layout`], or if the extras break a line, e.g. with a comment behind the
    /// opening bracket.
    pub fn bracketed(
        (open, close): (char, char),
        (leading, trailing): (Node, Node),
        items: Vec<Node>,
        max_items: usize,
        f: &FormatConfig,
    ) -> Node {
        let broken = matches!(
            BreakMode::from_layout(&items, max_items, f),
            BreakMode::WithIndent(_)
        ) || leading.has_line_break()
            || trailing.has_line_break();

        if broken {
            let items: Vec<Node> = items
                .into_iter()
                .map(|item| node!(item ',' Break::Hard))
                .collect();
            node!(
                open
                Node::indent(f.indent_width, node!(leading Break::Hard items trailing))
                Break::Hard
                close
            )
        } else {
            let separator = node!(',' Node::Column(Column::Separator) Break::Soft);
            node!(open leading Node::hlist(items, separator) trailing close)
        }
    }

    /// The width in characters, if the node is on one line.
    pub fn estimate_width(&self) -> usize {
        match &self {
            Node::Nil | Node::Column(_) | Node::Break(Break::Hard | Break::Paragraph) => 0,
            Node::Text(text) => text.chars().count(),
            Node::Break(Break::Soft) => 1,
            Node::Indent { width, node } => width + node.estimate_width(),
            Node::Group(group) => group.iter().map(Node::estimate_width).sum(),
        }
    }

    pub fn indent(width: usize, node: impl Into<Node>) -> Self {
        Node::Indent {
            width,
            node: Box::new(node.into()),
        }
    }

    /// Whether the node breaks a line, e.g. with a single line comment.
    pub fn has_line_break(&self) -> bool {
        match &self {
            Node::Text(text) => text.contains('\n'),
            Node::Break(line_break) => *line_break > Break::Soft,
            Node::Indent { node, .. } => node.has_line_break(),
            Node::Group(group) => group.iter().any(Node::has_line_break),
            Node::Nil | Node::Column(_) => false,
        }
    }

    /// Whether the node ends with a line break, e.g. behind a single line comment.
    pub fn ends_with_line_break(&self) -> bool {
        match &self {
            Node::Break(line_break) => *line_break > Break::Soft,
            Node::Indent { node, .. } => node.ends_with_line_break(),
            Node::Group(group) => group.last().is_some_and(Node::ends_with_line_break),
            Node::Nil | Node::Text(_) | Node::Column(_) => false,
        }
    }

    /// Compact nodes: flatten groups, remove `Nil` nodes, concatenate adjacent texts and merge
    /// adjacent breaks into the strongest one.
    pub fn compact(nodes: Vec<Node>) -> Self {
        fn flatten(nodes: Vec<Node>, flat: &mut Vec<Node>) {
            for node in nodes {
                match node {
                    Node::Group(group) => flatten(group, flat),
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
                (Some(Node::Break(last)), Node::Break(next)) => *last = (*last).max(next),
                (_, node) => compacted.push(node),
            }
        }

        match compacted.len() {
            0 => Node::Nil,
            1 => compacted.remove(0),
            _ => Node::Group(compacted),
        }
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

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut state = RenderState::default();
        self.render_recursive(&mut state);

        for (i, paragraph) in state.paragraphs.iter_mut().enumerate() {
            paragraph.align();
            if i > 0 {
                write!(f, "\n\n")?;
            }
            write!(f, "{paragraph}")?;
        }
        // The output ends with a line break.
        match state.pending {
            Some(Break::Hard | Break::Paragraph) => writeln!(f),
            _ => Ok(()),
        }
    }
}

/// The state of rendering nodes into paragraphs of lines.
#[derive(Default)]
struct RenderState {
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

    /// Write a text behind the pending break.
    fn write(&mut self, text: &str) {
        // No space in front of a separator, a closing bracket, an access or a space.
        let pending = match self.pending.take() {
            Some(Break::Soft) if text.starts_with([',', ';', ')', ']', '.', ' ']) => None,
            pending => pending,
        };
        let line = Line::new(self.indent_level);
        match pending {
            // No breaks at the start of the output.
            _ if self.paragraphs.is_empty() => {
                self.paragraphs.push(Paragraph { lines: vec![line] })
            }
            None => {}
            Some(Break::Soft) if self.no_space && !text.starts_with("//") => {}
            Some(Break::Soft) => {
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
        if matches!(self.pending, None | Some(Break::Soft))
            && let Some(line) = self.line()
        {
            line.mark(column);
        }
    }
}

impl Node {
    fn render_recursive(&self, state: &mut RenderState) {
        match self {
            Node::Text(text) => state.write(text),
            Node::Break(line_break) => state.pending = state.pending.max(Some(*line_break)),
            Node::Column(column) => state.mark(*column),
            Node::Group(group) => group.iter().for_each(|node| node.render_recursive(state)),
            Node::Indent { width, node } => {
                state.indent_level += width;
                node.render_recursive(state);
                state.indent_level -= width;
            }
            Node::Nil => {}
        }
    }
}
