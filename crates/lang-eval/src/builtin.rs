// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_builtin::{BuiltinFunction, BuiltinItem, BuiltinOperation, BuiltinPrimitive};
use mucad_lang_base::PushIssue;
use mucad_lang_types::{ArgumentValueList, ModelTree};

use crate::{
    ArgumentMatch, Callable, EvalContext, EvalError, EvalErrorKind, EvalResult,
    context::ContextScope,
};

impl Callable for BuiltinPrimitive {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        use crate::ArgumentMatch;

        let matches = self.match_arguments(args).map_err(|err| {
            EvalError::argument_match(context.current_symbol_src_ref(), self.info.item_name(), err)
        })?;
        let src_ref = context.current_symbol_src_ref();
        let mut models = Vec::new();
        for args in matches.arguments {
            let model = context.with_builtin_context(src_ref, |ctx| (self.f)(args, ctx))?;
            models.push(ModelTree::from(model));
        }

        Ok(ModelTree::to_replication(models).into())
    }
}

impl Callable for BuiltinFunction {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        let src_ref = context.current_symbol_src_ref();
        let name = context.current_symbol_name().unwrap_or_default();
        let matches = self
            .match_arguments(args)
            .map_err(|err| EvalError::argument_match(src_ref, name, err))?;

        Ok(matches
            .try_value(|args| context.with_builtin_context(src_ref, |ctx| self.call(args, ctx)))?)
    }
}

impl Callable for BuiltinOperation {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        let src_ref = context.current_symbol_src_ref();
        let name = context.current_symbol_name().unwrap_or_default();
        let matches = self
            .match_arguments(args)
            .map_err(|err| EvalError::argument_match(src_ref, name, err))?;
        let mut models = Vec::new();
        for args in matches.arguments {
            models.push(context.with_builtin_context(src_ref, |ctx| (self.f)(args, ctx))?);
        }

        Ok(ModelTree::to_replication(models).into())
    }
}

impl Callable for BuiltinItem {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        match self {
            // The inherent `BuiltinFunction::call` takes matched arguments.
            BuiltinItem::Function(f) => Callable::call(f, args, context),
            BuiltinItem::Primitive(p) => p.call(args, context),
            BuiltinItem::Operation(op) => op.call(args, context),
            BuiltinItem::Constant(_) | BuiltinItem::Module(_) => {
                context.catch(EvalErrorKind::SymbolCannotBeCalled {
                    path: context.current_symbol_name().unwrap_or_default(),
                    src_ref: context.current_symbol_src_ref(),
                })
            }
        }
    }
}
