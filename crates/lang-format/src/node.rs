// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{FormatConfig, node};

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
    /// front of a separator, a closing bracket, a `.` or a space.
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
            BreakMode::NoBreak => Self::hlist(nodes, node!(sep Break::Soft)),
            BreakMode::WithIndent(indent_width) => Self::vlist(nodes, sep, indent_width),
        }
    }

    /// The width in characters, if the node is on one line.
    pub fn estimate_width(&self) -> usize {
        match &self {
            Node::Nil | Node::Break(Break::Hard | Break::Paragraph) => 0,
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
            Node::Nil => false,
        }
    }

    /// Whether the node ends with a line break, e.g. behind a single line comment.
    pub fn ends_with_line_break(&self) -> bool {
        match &self {
            Node::Break(line_break) => *line_break > Break::Soft,
            Node::Indent { node, .. } => node.ends_with_line_break(),
            Node::Group(group) => group.last().is_some_and(Node::ends_with_line_break),
            Node::Nil | Node::Text(_) => false,
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
        self.render_recursive(f, &mut state)?;
        // The output ends with a line break.
        match state.pending {
            Some(Break::Hard | Break::Paragraph) => writeln!(f),
            _ => Ok(()),
        }
    }
}

#[derive(Default)]
struct RenderState {
    indent_level: usize,
    /// The strongest break since the last text, which is written in front of the next text.
    pending: Option<Break>,
    /// Nothing has been written yet.
    started: bool,
    /// The last text ends with a space, an opening bracket or a `.`, which has no space behind it.
    no_space: bool,
}

impl Node {
    fn render_recursive(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        state: &mut RenderState,
    ) -> std::fmt::Result {
        match self {
            Node::Text(text) => {
                // No space in front of a separator, a closing bracket, an access or a space.
                let pending = match state.pending.take() {
                    Some(Break::Soft) if text.starts_with([',', ';', ')', ']', '.', ' ']) => None,
                    pending => pending,
                };
                let indent = " ".repeat(state.indent_level);
                match pending {
                    // No breaks at the start of the output.
                    _ if !state.started => {}
                    None => {}
                    Some(Break::Soft) if state.no_space => {}
                    Some(Break::Soft) => write!(f, " ")?,
                    Some(Break::Hard) => write!(f, "\n{indent}")?,
                    Some(Break::Paragraph) => write!(f, "\n\n{indent}")?,
                }
                state.started = true;
                state.no_space = text.ends_with(['(', '[', '.', ' ']);
                write!(f, "{text}")
            }
            Node::Break(line_break) => {
                state.pending = state.pending.max(Some(*line_break));
                Ok(())
            }
            Node::Group(group) => group
                .iter()
                .try_for_each(|node| node.render_recursive(f, state)),
            Node::Indent { width, node } => {
                state.indent_level += width;
                node.render_recursive(f, state)?;
                state.indent_level -= width;
                Ok(())
            }
            Node::Nil => Ok(()),
        }
    }
}
