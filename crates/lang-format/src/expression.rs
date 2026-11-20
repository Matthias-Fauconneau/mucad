// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Break, Column, Format, Node, node};

use mucad_lang_parse::{ast, token::escape_char};

impl Format for ast::BinaryOperator {
    fn format(&self) -> Node {
        self.to_string().into()
    }
}

impl Format for ast::UnaryOperator {
    fn format(&self) -> Node {
        self.to_string().into()
    }
}

impl Format for ast::Body {
    fn format(&self) -> Node {
        format_body(self, true)
    }
}

/// A body, which is on one line, if it only has a tail without line breaks and `inline` is
/// set, e.g. `{ a }`.
///
/// Otherwise, the statements start a line, behind a comment on the line of the `{`.
fn format_body(body: &ast::Body, inline: bool) -> Node {
    let list = &body.statements;

    if list.statements.is_empty() {
        match &list.tail {
            None => {
                let extras = node!(list.extras => Node::Nil);
                // Without comments, an empty body is `{}`.
                return match extras {
                    Node::Nil | Node::Break(Break::Space) => "{}".into(),
                    extras => node!("{" Node::indent(extras) "}"),
                };
            }
            Some(tail) if inline => {
                let tail = node!(list.extras => tail.format());
                if !tail.has_line_break() {
                    return node!("{ " tail " }");
                }
            }
            Some(_) => {}
        }
    }

    node!(
        "{"
        Node::indent(node!(
            list.extras.leading
            Break::Hard
            list
            list.extras.trailing
        ))
        Break::Hard
        "}"
    )
}

impl Format for ast::Expression {
    fn format(&self) -> Node {
        match &self {
            ast::Expression::Literal(literal) => literal.format(),
            ast::Expression::Bracketed(bracket, _) => node!('(' bracket.format() ')'),
            ast::Expression::Tuple(tuple_expression) => tuple_expression.format(),
            ast::Expression::Range(range_expr) => range_expr.format(),
            ast::Expression::List(list_expr) => list_expr.format(),
            ast::Expression::String(format_string) => format_string.format(),
            ast::Expression::SymbolPath(qualified_name) => qualified_name.format(),
            ast::Expression::Marker(identifier) => format!("@{}", identifier.name).into(),
            ast::Expression::BinaryOperation(binary_operation) => binary_operation.format(),
            ast::Expression::UnaryOperation(unary_operation) => unary_operation.format(),
            ast::Expression::Body(body) => body.format(),
            ast::Expression::Call(call) => call.format(),
            ast::Expression::ElementAccess(element_access) => element_access.format(),
            ast::Expression::If(i) => i.format(),
            ast::Expression::Error(_) => Node::Nil,
        }
    }
}

impl Format for ast::StringPart {
    fn format(&self) -> Node {
        match &self {
            ast::StringPart::Char(string_character) => string_character.format(),
            ast::StringPart::Content(string_literal) =>
            // Simply clone the content of the string literal, because we do not want to have quotes '"'
            {
                string_literal.content.clone().into()
            }
            ast::StringPart::Expression(string_expression) => string_expression.format(),
        }
    }
}

impl Format for ast::StringCharacter {
    fn format(&self) -> Node {
        // An escaped character, e.g. `\/`, which is no escape sequence by `escape_char`.
        match escape_char(self.character) {
            Some(escaped) => escaped.into(),
            None => format!("\\{}", self.character).into(),
        }
    }
}

impl Format for ast::StringExpression {
    fn format(&self) -> Node {
        node!(self.extras =>
            '{' self.expr self.specification '}'
        )
    }
}

impl Format for ast::StringFormatSpecification {
    fn format(&self) -> Node {
        match (&self.width, &self.precision) {
            (Some(Ok(width)), Some(Ok(precision))) => format!(":0{width}.{precision}").into(),
            (None, Some(Ok(precision))) => format!(":.{precision}").into(),
            (Some(Ok(width)), None) => format!(":0{width}").into(),
            _ => Node::Nil,
        }
    }
}

impl Format for ast::FormatString {
    fn format(&self) -> Node {
        node!(self.extras =>
            '"'
            self.parts
                .iter()
                .map(|part| part.format())
                .collect::<Vec<_>>()
            '"'
        )
    }
}

impl Format for ast::TupleItem {
    fn format(&self) -> Node {
        node!(self.extras =>
            match &self.id {
                Some(name) => node!(name Node::Column(Column::Assign) " = " self.expr),
                None => node!(self.expr)
            }
        )
    }
}

impl Format for ast::TupleExpression {
    fn format(&self) -> Node {
        let nodes: Vec<Node> = self.values.iter().map(|item| item.format()).collect();
        Node::bracketed(("(", ")"), &self.extras, nodes)
    }
}

impl Format for ast::ListItem {
    fn format(&self) -> Node {
        node!(self.extras => self.expr)
    }
}

impl Format for ast::RangeExpression {
    fn format(&self) -> Node {
        // The extras are inside the brackets, a comment breaks the range like a list.
        let extras = &self.extras;
        node!(
            Node::group(node!(
                '['
                Node::indent(node!(
                    extras.leading Break::SoftLine self.start ".." self.end extras.trailing
                ))
                Break::SoftLine
                ']'
            ))
            self.unit
        )
    }
}

impl Format for ast::ListExpression {
    fn format(&self) -> Node {
        let nodes: Vec<Node> = self.items.iter().map(|item| item.format()).collect();
        node!(
            Node::bracketed(("[", "]"), &self.extras, nodes) self.unit
        )
    }
}

impl Format for ast::SymbolPath {
    fn format(&self) -> Node {
        node!(self.extras =>
            if self.prefix.is_some() { "::" } else { "" }
            Node::join(self.parts.iter().map(|identifier| identifier.format()), "::")
        )
    }
}

/// An operation is on one line, if it fits, otherwise its operator starts an indented line, e.g.:
///
/// ```mu
/// hole = Circle(d = 9mm).extrude(20mm).translate(z = RING_RADIUS - 2mm)
///     | Circle(d = 15mm).extrude(5mm).translate(z = HOUSING_RADIUS - 3mm);
/// ```
impl Format for ast::BinaryOperation {
    fn format(&self) -> Node {
        Node::group(node!(
            self.lhs
            Node::indent(node!(Break::Line self.op Break::Space self.rhs))
        ))
    }
}

impl Format for ast::UnaryOperation {
    fn format(&self) -> Node {
        node!(self.op self.rhs)
    }
}

impl Format for ast::Argument {
    fn format(&self) -> Node {
        match self {
            ast::Argument::Unnamed(arg) => arg.format(),
            ast::Argument::Named(arg) => arg.format(),
        }
    }
}

impl Format for ast::UnnamedArgument {
    fn format(&self) -> Node {
        node!(self.extras => self.expr)
    }
}

impl Format for ast::NamedArgument {
    fn format(&self) -> Node {
        node!(self.extras => self.id Node::Column(Column::Assign) " = " self.expr)
    }
}

impl Format for ast::ArgumentList {
    fn format(&self) -> Node {
        let nodes: Vec<Node> = self.arguments.iter().map(|item| item.format()).collect();
        Node::bracketed(("(", ")"), &self.extras, nodes)
    }
}

impl Format for ast::Call {
    fn format(&self) -> Node {
        node!(self.extras => self.path self.arguments)
    }
}

impl Format for ast::ElementInner {
    fn format(&self) -> Node {
        use ast::ElementInner::*;
        match &self {
            Attribute(identifier) => node!('#' identifier),
            Tuple(identifier) => node!('.' identifier),
            Method(call) => node!('.' call),
            ListElement(expression) => node!('[' expression ']'),
        }
    }
}

impl Format for ast::Element {
    fn format(&self) -> Node {
        node!(self.extras => self.inner)
    }
}

/// A chain with more than two method calls has each method call on its own line, e.g.:
///
/// ```mu
/// Circle(r = 5mm)
///     .extrude(5mm)
///     .translate(z = 2mm)
///     .center();
/// ```
///
/// Two method calls are on one line, if they fit and have no comment, a single method call
/// always is, and is only indented behind a comment.
///
/// The method calls are indented, unless the chain starts with a body, e.g.:
///
/// ```mu
/// {
///     Circle(r = 5mm);
/// }
/// .extrude(5mm)
/// .translate(z = 2mm)
/// .center();
/// ```
impl Format for ast::ElementAccess {
    fn format(&self) -> Node {
        /// The most method calls on one line.
        const MAX_METHODS: usize = 2;

        let methods = self
            .element_chain
            .iter()
            .filter(|element| matches!(element.inner, ast::ElementInner::Method(_)))
            .count();
        let line_break = match methods {
            0 | 1 => Node::Nil,
            2..=MAX_METHODS => Break::SoftLine.into(),
            _ => Break::Hard.into(),
        };

        let chain: Vec<Node> = self
            .element_chain
            .iter()
            .map(|element| match element.inner {
                ast::ElementInner::Method(_) => node!(line_break.clone() element),
                _ => element.format(),
            })
            .collect();
        // The chain is not indented without line breaks, e.g. not the arguments of a single call,
        // unless a comment breaks the line in front of it.
        let expr = self.expr.format();
        let chain = match self.expr.as_ref() {
            ast::Expression::Body(_) => node!(chain),
            _ if methods <= 1 && !expr.has_line_break() => node!(chain),
            _ => Node::indent(chain),
        };
        // Only the method calls are grouped, which fit behind the last line of the expression.
        node!(expr Node::group(chain))
    }
}

impl Format for ast::If {
    fn format(&self) -> Node {
        node!(self.extras =>
            "if " self.condition ' ' format_body(&self.body, false)
            match &self.else_body {
                Some(else_body) => node!(Break::Space "else " format_body(else_body, false)),
                None => match &self.next_if {
                    Some(_) => Node::Nil,
                    None => Break::Hard.into()
                }
            }
            match &self.next_if {
                Some(next_if) => node!(Break::Space "else " next_if),
                None => Node::Nil
            }
        )
    }
}
