// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The types of expressions, as far as they are known before evaluation.

use mucad_builtin::BuiltinLibrary;
use mucad_lang_base::{ExternalId, SymbolId};
use mucad_lang_lower::ir;
use mucad_lang_types::{Ty, Type};

use crate::{Library, LibraryCache, SymbolDef, SymbolNodeRef};

/// The depth to follow constants and aliases, which stops at cyclic definitions.
const MAX_DEPTH: usize = 16;

/// The symbols to derive types from: of a library, of the libraries it uses and the built-ins.
#[derive(Clone, Copy)]
pub struct TypeContext<'a> {
    lib: &'a Library,
    /// The other libraries, e.g. `std`. Without it, their symbols have no known types.
    lib_cache: Option<&'a LibraryCache>,
    builtins: &'a BuiltinLibrary,
    /// The types of the local variables, which are visible, see [`TypeContext::with_locals`].
    locals: Option<&'a Locals>,
    /// The depth of followed constants and aliases.
    depth: usize,
}

/// The types of the local variables of scopes, e.g. of the parameters and local assignments of
/// a function, which are known after their declaration.
#[derive(Debug, Default)]
pub struct Locals {
    /// The scopes, the innermost last, with the types of their locals.
    scopes: Vec<Vec<(String, Type)>>,
}

impl Locals {
    /// Open a scope, e.g. of a function body.
    pub fn push(&mut self) {
        self.scopes.push(Vec::new());
    }

    /// Close the innermost scope with its locals.
    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    /// Declare a local in the innermost scope, or give it a new type, e.g. of a new assignment.
    pub fn insert(&mut self, name: impl ToString, ty: Type) {
        let name = name.to_string();
        if let Some(scope) = self.scopes.last_mut() {
            scope.retain(|(local, _)| *local != name);
            scope.push((name, ty));
        }
    }

    /// The type of a visible local, the innermost one of its name.
    fn get(&self, name: &str) -> Option<&Type> {
        self.scopes
            .iter()
            .rev()
            .flat_map(|scope| scope.iter().rev())
            .find(|(local, _)| local == name)
            .map(|(_, ty)| ty)
    }
}

/// Derive the type of an expression, as far as it is known before evaluation.
pub trait DeriveType {
    /// The type of the expression or `Any` if it is unknown, e.g. of a local variable.
    ///
    /// `Any` matches every type, so an unknown type is never a type error.
    fn derive_type(&self, ctx: &TypeContext) -> Type;
}

impl<'a> TypeContext<'a> {
    /// A context for the symbols of `lib` and the built-ins.
    pub fn new(lib: &'a Library, builtins: &'a BuiltinLibrary) -> Self {
        Self {
            lib,
            lib_cache: None,
            builtins,
            locals: None,
            depth: 0,
        }
    }

    /// Derive the types of local variables, e.g. of a function, which is checked.
    pub fn with_locals<'b>(self, locals: &'b Locals) -> TypeContext<'b>
    where
        'a: 'b,
    {
        TypeContext {
            locals: Some(locals),
            ..self
        }
    }

    /// Derive the types of symbols of other libraries, e.g. of `std`.
    pub fn with_lib_cache(mut self, lib_cache: &'a LibraryCache) -> Self {
        self.lib_cache = Some(lib_cache);
        self
    }

    /// The type of the value of the symbol of `path`, e.g. of a constant.
    fn path_type(&self, path: &ir::Path) -> Type {
        if let Some(SymbolId::Local(name)) = path.symbol_id() {
            return (self.locals)
                .and_then(|locals| locals.get(name.as_ref()))
                .cloned()
                .unwrap_or_default();
        }
        self.with_symbol(path.symbol_id(), |symbol, ctx| match symbol {
            Symbol::Def(SymbolDef::Constant(constant)) if constant.ty.ty != Type::Any => {
                constant.ty.ty.clone()
            }
            Symbol::Def(SymbolDef::Constant(constant)) => constant.expr.derive_type(ctx),
            Symbol::Def(SymbolDef::Function(function)) => function.ty(),
            Symbol::Def(_) => Type::Any,
            Symbol::Builtin(item) => item.ty().unwrap_or_default(),
        })
    }

    /// The type of the result of a call, e.g. the return type of a function.
    ///
    /// A list argument for a parameter of its element type calls a function once per element,
    /// so its result is a list, e.g. `[Scalar]` of `cos([0°, 90°])`. An argument of unknown
    /// type might be such a list, so the result is unknown, too.
    fn call_type<Expr: ir::ExprSpec + DeriveType>(&self, call: &ir::Call<Expr>) -> Type {
        use mucad_builtin::BuiltinItem;

        // A list literal, e.g. `[1mm, 2mm]`, is a list of the common type of its elements.
        if call.path.symbol_id() == Some(&SymbolId::Builtin(mucad_builtin::__mu!(core::list))) {
            let mut types = call
                .args
                .args
                .iter()
                .map(|arg| arg.expr().derive_type(self));
            return match types.next() {
                Some(first) if first != Type::Any && types.all(|ty| ty == first) => {
                    Type::List(Box::new(first))
                }
                _ => Type::Any,
            };
        }

        let (ty, parameters) = self.with_symbol(call.path.symbol_id(), |symbol, _| match symbol {
            Symbol::Def(SymbolDef::Function(function)) => (
                function.return_type().unwrap_or_default(),
                function.signature.ty().call_ty.parameters,
            ),
            // A sketch is a 2D model, a part a 3D model, also of several calls.
            Symbol::Def(SymbolDef::Workbench(workbench)) => (workbench.signature.kind.ty(), None),
            Symbol::Def(_) => (Type::Any, None),
            // A built-in without return type, e.g. `debug::print`, returns nothing.
            Symbol::Builtin(item @ BuiltinItem::Function(function)) => (
                item.return_type().unwrap_or_default(),
                function.ty().call_ty.parameters,
            ),
            // The models of several calls of primitives and operations are a model.
            Symbol::Builtin(item) => (item.return_type().unwrap_or_default(), None),
        });
        let Some(parameters) = parameters else {
            return ty;
        };

        let expansions = call.args.args.iter().map(|arg| {
            let arg_ty = arg.expr().derive_type(self);
            // A named argument is matched by its name or an abbreviation of it, otherwise by type.
            let named = arg.name().map(|name| {
                parameters
                    .iter()
                    .filter(|(id, _)| id.to_string().starts_with(&name.to_string()))
                    .collect::<Vec<_>>()
            });
            let candidates = match named {
                Some(named) if !named.is_empty() => named,
                _ => parameters.iter().collect(),
            };
            expansion(&arg_ty, candidates.into_iter().map(|(_, ty)| ty))
        });
        match expansions.max().unwrap_or(Expansion::None) {
            Expansion::None => ty,
            Expansion::List => Type::List(Box::new(ty)),
            Expansion::Unknown => Type::Any,
        }
    }

    /// Run `f` with the symbol `id` stands for and a context of its library.
    ///
    /// Aliases are followed, e.g. `std::print` for `__mu::debug::print`.
    /// Locals and unknown symbols have the type `Any`.
    fn with_symbol<T: Default>(
        &self,
        id: Option<&SymbolId>,
        f: impl FnOnce(Symbol, &TypeContext) -> T,
    ) -> T {
        if self.depth > MAX_DEPTH {
            return T::default();
        }
        let ctx = Self {
            depth: self.depth + 1,
            ..*self
        };

        match id {
            Some(SymbolId::Item(node)) => match self.lib.symbol_id(*node, None) {
                SymbolId::Item(node) => match self.lib.arena.get(node) {
                    Some(node) => f(Symbol::Def(&node.get().def), &ctx),
                    None => T::default(),
                },
                // An alias of a symbol of another library or of a built-in.
                id => ctx.with_symbol(Some(&id), f),
            },
            Some(SymbolId::External(ExternalId { lib_id, id: node })) => {
                let Some(lib) = self.lib_cache.and_then(|lib_cache| lib_cache.get(lib_id)) else {
                    return T::default();
                };
                match lib.symbol_id(*node, Some(*lib_id)) {
                    SymbolId::External(ExternalId {
                        lib_id: alias_lib,
                        id,
                    }) if alias_lib == *lib_id => {
                        let node = SymbolNodeRef::new(id, &lib.arena);
                        // A context of the other library, which only lives while it is borrowed.
                        let lib_ctx = TypeContext {
                            lib: &lib,
                            lib_cache: ctx.lib_cache,
                            builtins: ctx.builtins,
                            // The locals are not visible in another library.
                            locals: None,
                            depth: ctx.depth,
                        };
                        f(Symbol::Def(&node.get().def), &lib_ctx)
                    }
                    // An alias of a symbol of a further library or of a built-in.
                    id => ctx.with_symbol(Some(&id), f),
                }
            }
            Some(SymbolId::Builtin(id)) => match self.builtins.get(*id) {
                Some(item) => f(Symbol::Builtin(item), &ctx),
                None => T::default(),
            },
            Some(SymbolId::Local(_)) | None => T::default(),
        }
    }
}

/// Whether an argument expands a call into one call per element of a list, see
/// `ArgumentMatch::match_arguments` of the evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Expansion {
    /// The argument is no list or matches a parameter of a list.
    None,
    /// The argument is a list, whose elements match a parameter.
    List,
    /// The type of the argument is unknown, it might be such a list.
    Unknown,
}

/// The expansion of an argument of a type, which matches one of the types of parameters.
///
/// A list, which matches a parameter itself, is passed as it is, e.g. to `Any`.
fn expansion<'a>(arg_ty: &Type, parameters: impl Iterator<Item = &'a Type> + Clone) -> Expansion {
    let mut parameters = parameters;
    match arg_ty {
        _ if parameters
            .clone()
            .any(|ty| arg_ty != &Type::Any && arg_ty.matches(ty)) =>
        {
            Expansion::None
        }
        Type::List(_) if parameters.any(|ty| arg_ty.matches_list_type(ty)) => Expansion::List,
        Type::Any if parameters.any(|ty| !matches!(ty, Type::Any | Type::List(_))) => {
            Expansion::Unknown
        }
        _ => Expansion::None,
    }
}

/// A symbol to derive a type from.
enum Symbol<'a> {
    /// A symbol of a library.
    Def(&'a SymbolDef),
    /// A built-in item.
    Builtin(&'static mucad_builtin::BuiltinItem),
}

/// The common type of the branches of an `if`, or `Any` if they differ or one is missing.
fn if_type<Expr: ir::ExprSpec>(
    if_: &ir::If<Expr>,
    body_type: &dyn Fn(&Expr::Body) -> Type,
) -> Type {
    let else_type = match (&if_.next_if, &if_.body_else) {
        (Some(next_if), _) => if_type(next_if, body_type),
        (None, Some(body_else)) => body_type(body_else),
        // Without `else`, the `if` has no value if its condition is false.
        (None, None) => return Type::Any,
    };
    match body_type(&if_.body) {
        ty if ty == else_type => ty,
        _ => Type::Any,
    }
}

impl DeriveType for ir::ConstantExpression {
    fn derive_type(&self, ctx: &TypeContext) -> Type {
        match self {
            ir::ConstantExpression::Value(value) => value.ty(),
            ir::ConstantExpression::Path(path) => ctx.path_type(path),
            ir::ConstantExpression::Call(call) => ctx.call_type(call),
            _ => Type::Any,
        }
    }
}

impl DeriveType for ir::WorkbenchExpression {
    fn derive_type(&self, ctx: &TypeContext) -> Type {
        match self {
            ir::WorkbenchExpression::Value(value) => value.ty(),
            ir::WorkbenchExpression::Path(path) => ctx.path_type(path),
            ir::WorkbenchExpression::Call(call) => ctx.call_type(call),
            // The model of a group depends on its statements.
            ir::WorkbenchExpression::If(if_) => if_type(if_, &|_| Type::Any),
            ir::WorkbenchExpression::Group(_) | ir::WorkbenchExpression::Marker(_) => Type::Any,
            _ => Type::Any,
        }
    }
}

impl DeriveType for ir::FunctionExpression {
    fn derive_type(&self, ctx: &TypeContext) -> Type {
        match self {
            ir::FunctionExpression::Value(value) => value.ty(),
            ir::FunctionExpression::Path(path) => ctx.path_type(path),
            ir::FunctionExpression::Call(call) => ctx.call_type(call),
            ir::FunctionExpression::Scope(scope) => scope.derive_type(ctx),
            ir::FunctionExpression::If(if_) => if_type(if_, &|scope| scope.derive_type(ctx)),
            _ => Type::Any,
        }
    }
}

/// The type of the value of a scope: of its last statement, e.g. a calculation, an `if` or a
/// `return`.
impl DeriveType for ir::function::Scope {
    fn derive_type(&self, ctx: &TypeContext) -> Type {
        match self.statements.last() {
            Some(statement) => statement.derive_type(ctx),
            None => Type::Any,
        }
    }
}

/// The type of the value of a statement, if it is the last one of a scope.
impl DeriveType for ir::FunctionStatement {
    fn derive_type(&self, ctx: &TypeContext) -> Type {
        match self {
            ir::FunctionStatement::Tail(expr) => expr.derive_type(ctx),
            ir::FunctionStatement::Call(call) => ctx.call_type(call),
            ir::FunctionStatement::If(if_) => if_type(if_, &|scope| scope.derive_type(ctx)),
            ir::FunctionStatement::Scope(scope) => scope.derive_type(ctx),
            ir::FunctionStatement::Return(ir::function::ReturnStatement {
                expr: Some(expr),
                ..
            }) => expr.derive_type(ctx),
            ir::FunctionStatement::Return(_) | ir::FunctionStatement::Local(_) => Type::Any,
            _ => Type::Any,
        }
    }
}
