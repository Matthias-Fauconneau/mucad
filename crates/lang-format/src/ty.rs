// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Format, Node, node};

use mucad_lang_parse::ast;

impl Format for ast::Unit {
    fn format(&self) -> Node {
        self.name.clone().into()
    }
}

impl Format for ast::Type {
    fn format(&self) -> Node {
        match &self {
            ast::Type::Single(single_type) => single_type.format(),
            ast::Type::List(list_type) => list_type.format(),
            ast::Type::Tuple(tuple_type) => tuple_type.format(),
        }
    }
}

impl Format for ast::SingleType {
    fn format(&self) -> Node {
        self.name.clone().into()
    }
}

impl Format for ast::ListType {
    fn format(&self) -> Node {
        node!('[' self.inner ']')
    }
}

impl Format for ast::TupleType {
    fn format(&self) -> Node {
        let nodes: Vec<Node> = self
            .inner
            .iter()
            .map(|item| match &item.0 {
                // A named tuple type has the types of its names, e.g. `(x: Length, y: Length)`.
                Some(name) => node!(name ": " item.1),
                None => item.1.format(),
            })
            .collect();
        Node::bracketed(("(", ")"), &Default::default(), nodes)
    }
}
