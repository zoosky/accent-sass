use crate::ast::SassMixin;
use crate::value::SassModule;
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

/// The configuration a `$with` argument describes: every key names a
/// `!default` variable of the file being loaded. `null` means none, and a
/// value that is not a map is an error. Shared by `meta.load` and
/// `meta.load-css`, which validate the result the way `@use ... with` does.
fn configuration_from_with(with: Value, span: Span) -> SassResult<Rc<RefCell<Configuration>>> {
    let with = match with {
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

    Ok(Rc::new(RefCell::new(configuration)))
}

/// Loads the module `$url` names, configured by `$with`, for `meta.load` and
/// `meta.load-css`. The two take the same arguments and differ only in what
/// they do with the module.
fn load_module_arg(
    args: &mut ArgumentResult,
    visitor: &mut Visitor,
) -> SassResult<Arc<RefCell<Module>>> {
    args.max_args(2)?;

    let span = args.span();

    let url = args
        .get_err(0, "url")?
        .assert_string_with_name("url", span)?
        .0;

    let configuration = configuration_from_with(args.default_arg(1, "with", Value::Null), span)?;

    let module = visitor.load_module_for_sass_script(&url, Rc::clone(&configuration), span)?;

    // Anything left over names a variable the loaded file does not declare with
    // `!default`, which is a mistake worth reporting rather than ignoring.
    Visitor::assert_configuration_is_empty(&configuration, true)?;

    Ok(module)
}

/// `meta.load($url, $with: null)`: loads a module as `@use` would and returns
/// it as a value, without emitting its CSS.
///
/// The file executes once, shares its state with any `@use` of it, and keeps
/// the CSS it emitted on record: `meta.css` emits a copy where it is included,
/// and a later `@use` of the module emits it there. Extension and
/// serialization errors in that CSS wait until it is emitted.
fn load(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    let module = load_module_arg(&mut args, visitor)?;

    Ok(Value::ModuleRef(SassModule::new(module)))
}

/// `meta.load-css($url, $with: null)`: loads a stylesheet for its CSS alone.
///
/// Defined as `meta.css(meta.load($url, $with))`, which is how dart-sass
/// defines it: the loaded file gets its own environment, so nothing it defines
/// is visible to the caller; only its CSS is, and it appears where the
/// `@include` was written.
fn load_css(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<()> {
    let module = load_module_arg(&mut args, visitor)?;

    visitor.emit_module_css(&module)
}

/// `meta.css($module)`: emits a copy of the module's CSS, upstream modules
/// included, where the mixin is included.
fn css(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<()> {
    args.max_args(1)?;

    let span = args.span();
    let module = module_from_value(args.get_err(0, "module")?, visitor, span)?;

    visitor.emit_module_css(&module)
}

/// `meta.get-module($module)`: the module behind a namespace, as a value.
fn get_module(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let span = args.span();
    let module = module_from_value(args.get_err(0, "module")?, visitor, span)?;

    Ok(Value::ModuleRef(SassModule::new(module)))
}

/// The module a `$module` argument names.
///
/// Since dart-sass 1.105.0 this is either a namespace string, looked up as
/// written, or a module reference, which stands for itself. Anything else is
/// the error dart-sass prints.
pub(crate) fn module_from_value(
    value: Value,
    visitor: &Visitor,
    span: Span,
) -> SassResult<Arc<RefCell<Module>>> {
    match value {
        Value::String(name, ..) => (*visitor.env.modules).borrow().get_as_written(&name, span),
        Value::ModuleRef(module) => Ok(Arc::clone(module.inner())),
        v => Err((
            format!(
                "$module: {} is neither a string nor a module reference.",
                v.inspect(span)?
            ),
            span,
        )
            .into()),
    }
}

/// [`module_from_value`] for a `$module: null` parameter, where `null` means
/// the caller's own scope.
pub(crate) fn optional_module_from_value(
    value: Value,
    visitor: &Visitor,
    span: Span,
) -> SassResult<Option<Arc<RefCell<Module>>>> {
    match value {
        Value::Null => Ok(None),
        v => module_from_value(v, visitor, span).map(Some),
    }
}

fn module_functions(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let span = args.span();
    let module = module_from_value(args.get_err(0, "module")?, visitor, span)?;
    let functions = (*module).borrow().functions(span);

    Ok(Value::Map(functions))
}

fn module_variables(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let span = args.span();
    let module = module_from_value(args.get_err(0, "module")?, visitor, span)?;
    let variables = (*module).borrow().variables(span);

    Ok(Value::Map(variables))
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
                CalculationArg::Operation { .. }
                | CalculationArg::Space(..)
                | CalculationArg::Paren(..) => Value::String(
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

    let module =
        optional_module_from_value(args.default_arg(1, "module", Value::Null), visitor, span)?;

    let mixin = match module {
        Some(module) => (*module).borrow().get_mixin(Spanned { node: name, span })?,
        None => visitor.env.get_mixin(Spanned { node: name, span }, None)?,
    };

    Ok(Value::MixinRef(SassMixin::new(mixin)))
}

/// `meta.module-mixins($module)`: every mixin the module exposes.
fn module_mixins(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let span = args.span();
    let module = module_from_value(args.get_err(0, "module")?, visitor, span)?;
    let mixins = (*module).borrow().mixins(span);

    Ok(Value::Map(mixins))
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
    f.insert_builtin("get-module", get_module);
    f.insert_builtin("load", load);

    f.insert_builtin_mixin("css", css, false);
    f.insert_builtin_mixin("load-css", load_css, false);
    f.insert_builtin_mixin("apply", apply, true);
}
