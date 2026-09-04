use crate::ast::SassMixin;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::ast::{Configuration, ConfiguredValue};
use crate::builtin::builtin_imports::*;

use crate::builtin::{
    meta::{
        call, content_exists, feature_exists, function_exists, get_function,
        global_variable_exists, inspect, keywords, mixin_exists, type_of, variable_exists,
    },
    modules::Module,
};
use crate::serializer::serialize_calculation_arg;

/// `meta.load-css($url, $with: null)`: loads a stylesheet for its CSS alone.
///
/// The loaded file gets its own environment, so nothing it defines is visible
/// to the caller; only its CSS is, and it appears where the `@include` was
/// written. `$with` configures the file's `!default` variables and is validated
/// the same way `@use ... with` validates its own.
fn load_css(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<()> {
    args.max_args(2)?;

    let span = args.span();

    let url = args
        .get_err(0, "url")?
        .assert_string_with_name("url", span)?
        .0;

    let with = match args.default_arg(1, "with", Value::Null) {
        Value::Map(map) => Some(map),
        Value::List(v, ..) if v.is_empty() => Some(SassMap::new()),
        Value::ArgList(v) if v.is_empty() => Some(SassMap::new()),
        Value::Null => None,
        v => return Err((format!("$with: {} is not a map.", v.inspect(span)?), span).into()),
    };

    let configuration = match with {
        None => Configuration::empty(),
        Some(with) => {
            let mut values = BTreeMap::new();

            for (key, value) in with {
                let name = Identifier::from(key.node.assert_string_with_name("with key", span)?.0);

                if values.contains_key(&name) {
                    return Err((
                        format!("The variable ${name} was configured twice.", name = name),
                        key.span,
                    )
                        .into());
                }

                values.insert(name, ConfiguredValue::explicit(value, span));
            }

            Configuration::explicit(values, span)
        }
    };

    let configuration = Rc::new(RefCell::new(configuration));

    visitor.load_css_module(url.as_ref(), Rc::clone(&configuration), span)?;

    // Anything left over names a variable the loaded file does not declare with
    // `!default`, which is a mistake worth reporting rather than ignoring.
    Visitor::assert_configuration_is_empty(&configuration, true)?;

    Ok(())
}

fn module_functions(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let module = Identifier::from(
        args.get_err(0, "module")?
            .assert_string_with_name("module", args.span())?
            .0,
    );

    Ok(Value::Map(
        (*(*visitor.env.modules).borrow().get(module, args.span())?)
            .borrow()
            .functions(args.span()),
    ))
}

fn module_variables(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let module = Identifier::from(
        args.get_err(0, "module")?
            .assert_string_with_name("module", args.span())?
            .0,
    );

    Ok(Value::Map(
        (*(*visitor.env.modules).borrow().get(module, args.span())?)
            .borrow()
            .variables(args.span()),
    ))
}

fn calc_args(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let calc = match args.get_err(0, "calc")? {
        Value::Calculation(calc) => calc,
        v => {
            return Err((
                format!("$calc: {} is not a calculation.", v.inspect(args.span())?),
                args.span(),
            )
                .into());
        }
    };

    let args = calc
        .args
        .into_iter()
        .map(|arg| {
            Ok(match arg {
                CalculationArg::Number(num) => Value::Dimension(num),
                CalculationArg::Calculation(calc) => Value::Calculation(calc),
                CalculationArg::String(s) | CalculationArg::Interpolation(s) => {
                    Value::String(s, QuoteKind::None)
                }
                CalculationArg::Operation { .. } | CalculationArg::Space(..) => Value::String(
                    serialize_calculation_arg(&arg, visitor.options, args.span())?,
                    QuoteKind::None,
                ),
            })
        })
        .collect::<SassResult<Vec<_>>>()?;

    Ok(Value::List(args, ListSeparator::Comma, Brackets::None))
}

fn get_mixin(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;

    let span = args.span();

    let name = Identifier::from(
        args.get_err(0, "name")?
            .assert_string_with_name("name", span)?
            .0,
    );

    let module = match args.default_arg(1, "module", Value::Null) {
        Value::String(s, ..) => Some(Spanned {
            node: Identifier::from(s),
            span,
        }),
        Value::Null => None,
        v => {
            return Err((
                format!("$module: {} is not a string.", v.inspect(span)?),
                span,
            )
                .into());
        }
    };

    let mixin = visitor
        .env
        .get_mixin(Spanned { node: name, span }, module)?;

    Ok(Value::MixinRef(SassMixin::new(mixin)))
}

/// `meta.module-mixins($module)`: every mixin the module exposes.
fn module_mixins(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let module = Identifier::from(
        args.get_err(0, "module")?
            .assert_string_with_name("module", args.span())?
            .0,
    );

    Ok(Value::Map(
        (*(*visitor.env.modules).borrow().get(module, args.span())?)
            .borrow()
            .mixins(args.span()),
    ))
}

/// `meta.accepts-content($mixin)`: whether the mixin takes a `@content` block.
fn accepts_content(mut args: ArgumentResult, _visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let span = args.span();
    let mixin = args.get_err(0, "mixin")?;

    Ok(Value::bool(
        mixin.assert_mixin("mixin", span)?.accepts_content(),
    ))
}

/// `meta.apply($mixin, $args...)`: includes a first-class mixin.
///
/// This is a mixin rather than a function so that it can appear where
/// `@include` does and carry a `@content` block, which it forwards to the mixin
/// it applies.
fn apply(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<()> {
    let span = args.span();

    let mixin = args.get_err(0, "mixin")?;
    let mixin = mixin.assert_mixin("mixin", span)?.clone();

    // Whatever is left over is the applied mixin's own argument list.
    let rest = args.into_remaining_arguments();

    visitor.apply_mixin(mixin, rest, span)
}

fn calc_name(mut args: ArgumentResult, _visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let calc = match args.get_err(0, "calc")? {
        Value::Calculation(calc) => calc,
        v => {
            return Err((
                format!("$calc: {} is not a calculation.", v.inspect(args.span())?),
                args.span(),
            )
                .into());
        }
    };

    Ok(Value::String(calc.name.to_string(), QuoteKind::Quoted))
}

pub(crate) fn declare(f: &mut Module) {
    f.insert_builtin("feature-exists", feature_exists);
    f.insert_builtin("inspect", inspect);
    f.insert_builtin("type-of", type_of);
    f.insert_builtin("keywords", keywords);
    f.insert_builtin("global-variable-exists", global_variable_exists);
    f.insert_builtin("variable-exists", variable_exists);
    f.insert_builtin("function-exists", function_exists);
    f.insert_builtin("mixin-exists", mixin_exists);
    f.insert_builtin("content-exists", content_exists);
    f.insert_builtin("module-variables", module_variables);
    f.insert_builtin("module-functions", module_functions);
    f.insert_builtin("get-function", get_function);
    f.insert_builtin("get-mixin", get_mixin);
    f.insert_builtin("module-mixins", module_mixins);
    f.insert_builtin("accepts-content", accepts_content);
    f.insert_builtin("call", call);
    f.insert_builtin("calc-args", calc_args);
    f.insert_builtin("calc-name", calc_name);

    f.insert_builtin_mixin("load-css", load_css, false);
    f.insert_builtin_mixin("apply", apply, true);
}
