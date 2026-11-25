// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Format, Node, node};

use mucad_lang_parse::{ast, token::escape_char};

impl Format for ast::Literal {
    fn format(&self) -> Node {
        node!(self.extras => self.literal)
    }
}

impl Format for ast::LiteralKind {
    fn format(&self) -> Node {
        match self {
            ast::LiteralKind::String(lit) => lit.format(),
            ast::LiteralKind::Bool(lit) => lit.format(),
            ast::LiteralKind::Integer(lit) => lit.format(),
            ast::LiteralKind::Float(lit) => lit.format(),
            ast::LiteralKind::Quantity(lit) => lit.format(),
            ast::LiteralKind::Error(_) => Node::Nil,
        }
    }
}

impl Format for ast::StringLiteral {
    fn format(&self) -> Node {
        string_literal(self, true)
    }
}

/// A string literal, which is split into pieces behind its spaces, if `split` is set, at which
/// it is continued with `+` on the next line, if it does not fit, see [`Node::Str`].
pub(crate) fn string_literal(literal: &ast::StringLiteral, split: bool) -> Node {
    let mut escaped = String::new();
    for c in literal.content.chars() {
        match escape_char(c) {
            Some(sequence) => escaped.push_str(sequence),
            None => escaped.push(c),
        }
    }
    match split {
        true => Node::Str(escaped.split_inclusive(' ').map(Node::from).collect()),
        false => Node::Str(vec![escaped.into()]),
    }
}

impl Format for ast::BoolLiteral {
    fn format(&self) -> Node {
        if self.value { "true" } else { "false" }.into()
    }
}

impl Format for ast::IntegerLiteral {
    fn format(&self) -> Node {
        self.value.clone().into()
    }
}

impl Format for ast::FloatLiteral {
    fn format(&self) -> Node {
        self.value.clone().into()
    }
}

impl Format for ast::QuantityLiteral {
    fn format(&self) -> Node {
        format!("{}{}", self.value, self.unit.name).into()
    }
}
