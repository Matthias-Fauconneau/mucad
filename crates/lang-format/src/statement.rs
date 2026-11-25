// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Break, Column, Format, Node, node};

use mucad_lang_parse::ast;

/// The visibility in front of a definition, e.g. `pub `.
impl Format for mucad_lang_base::Spanned<ast::def::Visibility> {
    fn format(&self) -> Node {
        match self.value {
            ast::def::Visibility::Public => "pub ".into(),
        }
    }
}

impl Format for ast::Parameter {
    fn format(&self) -> Node {
        node!(self.extras =>
            match (&self.ty, &self.default) {
                (None, None) => self.id.format(),
                (None, Some(def)) => node!(self.id Node::Column(Column::Assign) " = " def),
                (Some(ty), None) => node!(self.id ": " ty),
                (Some(ty), Some(def)) => node!(self.id ": " ty Node::Column(Column::Assign) " = " def),
            }
        )
    }
}

impl Format for ast::ParameterList {
    fn format(&self) -> Node {
        let nodes: Vec<Node> = self.parameters.iter().map(|item| item.format()).collect();
        Node::bracketed(("(", ")"), &self.extras, nodes)
    }
}

impl Format for ast::def::WorkbenchKind {
    fn format(&self) -> Node {
        self.to_string().into()
    }
}

impl Format for ast::def::Workbench {
    fn format(&self) -> Node {
        node!(self.extras =>
            self.doc
            self.attr
            self.vis self.kind ' ' self.id
            self.parameters ' '
            self.body
        )
    }
}

impl Format for ast::def::InlineModule {
    fn format(&self) -> Node {
        node!(self.extras =>
            self.doc
            self.attr
            self.vis "mod " self.id ' '
            self.body
        )
    }
}

impl Format for ast::def::FileModule {
    fn format(&self) -> Node {
        node!(self.extras =>
            self.doc
            self.attr
            self.vis "mod " self.id
        )
    }
}

impl Format for ast::def::Function {
    fn format(&self) -> Node {
        let return_type = match &self.return_type {
            Some(ty) => node!("-> " ty " "),
            None => Node::Nil,
        };

        node!(self.extras =>
            self.doc
            self.attr
            self.vis "fn " self.id self.parameters " " return_type
            self.body
        )
    }
}

impl Format for ast::def::UseStatementPart {
    fn format(&self) -> Node {
        match &self {
            Self::Identifier(identifier) => identifier.format(),
            Self::Glob(_) => '*'.into(),
            Self::Error(_) => Node::Nil,
        }
    }
}

impl Format for ast::def::UseName {
    fn format(&self) -> Node {
        node!(self.extras =>
            if self.prefix.is_some() { "::" } else { "" }
            Node::join(self.parts.iter().map(|part| part.format()), "::")
        )
    }
}

impl Format for ast::def::Use {
    fn format(&self) -> Node {
        node!(self.extras =>
            self.attr
            self.vis "use " self.name
            self.use_as.as_ref().map(|ident| node!(" as " ident))
        )
    }
}

impl Format for ast::def::Constant {
    fn format(&self) -> Node {
        node!(self.extras =>
            self.doc
            self.attr
            self.vis "const " self.id Node::Column(Column::Assign) " = " self.expr
        )
    }
}

impl Format for ast::Init {
    fn format(&self) -> Node {
        node!(self.extras =>
            self.doc
            self.attr
            "init" self.parameters Break::Space self.body
        )
    }
}

impl Format for ast::Return {
    fn format(&self) -> Node {
        node!(self.extras =>
            "return"
            match &self.expr {
                Some(value) => node!(Break::Space value),
                None => Node::Nil
            }
        )
    }
}

impl Format for ast::LocalAssignment {
    fn format(&self) -> Node {
        let assignment = node!(
            self.id
            match &self.ty {
                Some(ty) => node!(':' Break::Space ty),
                None => Node::Nil,
            }
            Node::Column(Column::Assign) Break::Space '=' Break::Space
        );

        node!(self.extras =>
            self.attr
            assignment self.expr
        )
    }
}

impl Format for ast::PropertyAssignment {
    fn format(&self) -> Node {
        node!(self.extras =>
            self.doc
            self.attr
            "prop" Break::Space self.id
            self.ty.as_ref().map(|ty| node!(':' Break::Space ty))
            Node::Column(Column::Assign) " = " self.value
        )
    }
}

impl Format for ast::ExpressionStatement {
    fn format(&self) -> Node {
        node!(self.extras =>
            self.attr
            self.expr
        )
    }
}

impl Format for ast::InnerDocComment {
    fn format(&self) -> Node {
        Node::comment(self.line.as_str(), None)
    }
}

impl Format for ast::AttributeCommand {
    fn format(&self) -> Node {
        match &self {
            ast::AttributeCommand::Ident(identifier) => identifier.format(),
            ast::AttributeCommand::Assignment(local_assignment) => local_assignment.format(),
            ast::AttributeCommand::Call(call) => call.format(),
        }
    }
}

impl Format for ast::Attribute {
    fn format(&self) -> Node {
        let prefix = if self.is_inner { "#![" } else { "#[" };

        let nodes: Vec<Node> = self.commands.iter().map(|attr| attr.format()).collect();
        node!(self.extras =>
            Node::bracketed((prefix, "]"), &Default::default(), nodes)
        )
    }
}

impl Format for Vec<ast::Attribute> {
    fn format(&self) -> Node {
        if self.is_empty() {
            Node::Nil
        } else {
            self.iter()
                .map(|attr| node!(attr Break::Hard))
                .collect::<Vec<_>>()
                .into()
        }
    }
}

impl Format for ast::Attributes {
    fn format(&self) -> Node {
        self.0.format()
    }
}

impl Format for ast::Statement {
    fn format(&self) -> Node {
        match &self {
            Self::Workbench(workbench_definition) => workbench_definition.format(),
            Self::InlineModule(inline_module) => inline_module.format(),
            Self::FileModule(file_module) => file_module.format(),
            Self::Function(function_definition) => function_definition.format(),
            Self::InnerDocComment(comment) => comment.format(),
            Self::Use(use_statement) => use_statement.format(),
            Self::Const(const_assignment) => const_assignment.format(),
            Self::Init(init_definition) => init_definition.format(),
            Self::Return(r) => r.format(),
            Self::InnerAttribute(attribute) => attribute.format(),
            Self::LocalAssignment(local_assignment) => local_assignment.format(),
            Self::Property(property_assignment) => property_assignment.format(),
            Self::Expression(expression_statement) => expression_statement.format(),
            Self::Error(_) => Node::Nil,
        }
    }
}

/// The statements, each on its own line, and the tail, without the extras of the list.
impl Format for ast::StatementList {
    fn format(&self) -> Node {
        let statements: Vec<Node> = self
            .statements
            .iter()
            .map(|(statement, extras)| {
                let semicolon = statement.ends_with_semicolon().then_some(';');
                node!(statement semicolon extras Break::Hard)
            })
            .collect();
        node!(statements self.tail.as_ref().map(|tail| tail.format()))
    }
}
