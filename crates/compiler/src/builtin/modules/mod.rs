use crate::ast::SassMixin;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use codemap::{Span, Spanned};

use crate::{
    ast::{ArgumentResult, AstForwardRule, BuiltinMixin, Mixin},
    builtin::Builtin,
    common::Identifier,
    error::SassResult,
    evaluate::{Environment, Visitor},
    selector::ExtensionStore,
    utils::{
        BaseMapView, LimitedMapView, MapView, MergedMapView, PrefixedMapView, PublicMemberMapView,
    },
    value::{SassFunction, SassMap, Value},
};

use super::builtin_imports::QuoteKind;
use crate::utils::{is_name, is_name_start};

pub(crate) use meta::optional_module_from_value;

mod color;
mod list;
mod map;
mod math;
mod meta;
mod selector;
mod string;

/// A [Module] that only exposes members that aren't shadowed by a given
/// blocklist of member names.
#[derive(Debug, Clone)]
pub(crate) struct ShadowedModule {
    #[allow(dead_code)]
    inner: Arc<RefCell<Module>>,
    scope: ModuleScope,
}

impl ShadowedModule {
    pub fn new(
        module: Arc<RefCell<Module>>,
        variables: Option<&HashSet<Identifier>>,
        functions: Option<&HashSet<Identifier>>,
        mixins: Option<&HashSet<Identifier>>,
    ) -> Self {
        let module_scope = module.borrow().scope();

        let variables = Self::shadowed_map(Arc::clone(&module_scope.variables), variables);
        let functions = Self::shadowed_map(Arc::clone(&module_scope.functions), functions);
        let mixins = Self::shadowed_map(Arc::clone(&module_scope.mixins), mixins);

        let new_scope = ModuleScope {
            variables,
            functions,
            mixins,
        };

        Self {
            inner: module,
            scope: new_scope,
        }
    }

    /// Whether a configuration naming `variables` could have configured the
    /// shadowed module: only names this view still exposes count.
    fn could_have_been_configured(&self, variables: &HashSet<Identifier>) -> bool {
        let visible: HashSet<Identifier> = self
            .scope
            .variables
            .keys()
            .into_iter()
            .filter(|name| variables.contains(name))
            .collect();

        (*self.inner).borrow().could_have_been_configured(&visible)
    }

    fn needs_blocklist<V: fmt::Debug + Clone>(
        map: Arc<dyn MapView<Value = V>>,
        blocklist: Option<&HashSet<Identifier>>,
    ) -> bool {
        blocklist.is_some()
            && !map.is_empty()
            && blocklist.unwrap().iter().any(|key| map.contains_key(*key))
    }

    fn shadowed_map<V: fmt::Debug + Clone + 'static>(
        map: Arc<dyn MapView<Value = V>>,
        blocklist: Option<&HashSet<Identifier>>,
    ) -> Arc<dyn MapView<Value = V>> {
        match blocklist {
            Some(..) if !Self::needs_blocklist(Arc::clone(&map), blocklist) => map,
            Some(blocklist) => Arc::new(LimitedMapView::blocklist(map, blocklist)),
            None => map,
        }
    }

    pub fn if_necessary(
        module: Arc<RefCell<Module>>,
        variables: Option<&HashSet<Identifier>>,
        functions: Option<&HashSet<Identifier>>,
        mixins: Option<&HashSet<Identifier>>,
    ) -> Option<Arc<RefCell<Module>>> {
        let module_scope = module.borrow().scope();

        let needs_blocklist = Self::needs_blocklist(Arc::clone(&module_scope.variables), variables)
            || Self::needs_blocklist(Arc::clone(&module_scope.functions), functions)
            || Self::needs_blocklist(Arc::clone(&module_scope.mixins), mixins);

        if needs_blocklist {
            Some(Arc::new(RefCell::new(Module::Shadowed(Self::new(
                module, variables, functions, mixins,
            )))))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ForwardedModule {
    scope: ModuleScope,
    #[allow(dead_code)]
    inner: Arc<RefCell<Module>>,
    #[allow(dead_code)]
    forward_rule: AstForwardRule,
}

impl ForwardedModule {
    pub fn new(module: Arc<RefCell<Module>>, rule: AstForwardRule) -> Self {
        let scope = (*module).borrow().scope();

        let variables = Self::forwarded_map(
            scope.variables,
            rule.prefix.as_deref(),
            rule.shown_variables.as_ref(),
            rule.hidden_variables.as_ref(),
        );

        let functions = Self::forwarded_map(
            scope.functions,
            rule.prefix.as_deref(),
            rule.shown_mixins_and_functions.as_ref(),
            rule.hidden_mixins_and_functions.as_ref(),
        );

        let mixins = Self::forwarded_map(
            scope.mixins,
            rule.prefix.as_deref(),
            rule.shown_mixins_and_functions.as_ref(),
            rule.hidden_mixins_and_functions.as_ref(),
        );

        let scope = ModuleScope {
            variables,
            mixins,
            functions,
        };

        ForwardedModule {
            inner: module,
            forward_rule: rule,
            scope,
        }
    }

    fn forwarded_map<T: Clone + fmt::Debug + 'static>(
        mut map: Arc<dyn MapView<Value = T>>,
        prefix: Option<&str>,
        safelist: Option<&HashSet<Identifier>>,
        blocklist: Option<&HashSet<Identifier>>,
    ) -> Arc<dyn MapView<Value = T>> {
        debug_assert!(safelist.is_none() || blocklist.is_none());

        if prefix.is_none() && safelist.is_none() && blocklist.is_none() {
            return map;
        }

        if let Some(prefix) = prefix {
            map = Arc::new(PrefixedMapView(map, prefix.to_owned()));
        }

        // `show` and `hide` were parsed and then ignored, so a hidden member
        // stayed reachable through the forwarding module.
        if let Some(safelist) = safelist {
            map = Arc::new(LimitedMapView::safelist(map, safelist));
        } else if let Some(blocklist) = blocklist {
            map = Arc::new(LimitedMapView::blocklist(map, blocklist));
        }

        map
    }

    /// Whether a configuration naming `variables` could have configured the
    /// forwarded module, seen through this forward's prefix and `show`/`hide`
    /// lists.
    fn could_have_been_configured(&self, variables: &HashSet<Identifier>) -> bool {
        let rule = &self.forward_rule;

        if rule.prefix.is_none()
            && rule.shown_variables.is_none()
            && rule.hidden_variables.as_ref().is_none_or(HashSet::is_empty)
        {
            return (*self.inner).borrow().could_have_been_configured(variables);
        }

        let mut names: HashSet<Identifier> = match &rule.prefix {
            Some(prefix) => variables
                .iter()
                .filter_map(|name| {
                    name.as_str()
                        .strip_prefix(prefix.as_str())
                        .map(Identifier::from)
                })
                .collect(),
            None => variables.clone(),
        };

        if let Some(shown) = &rule.shown_variables {
            names.retain(|name| shown.contains(name));
        } else if let Some(hidden) = &rule.hidden_variables
            && !hidden.is_empty()
        {
            names.retain(|name| !hidden.contains(name));
        }

        (*self.inner).borrow().could_have_been_configured(&names)
    }

    pub fn if_necessary(
        module: Arc<RefCell<Module>>,
        rule: AstForwardRule,
    ) -> Arc<RefCell<Module>> {
        if rule.prefix.is_none()
            && rule.shown_mixins_and_functions.is_none()
            && rule.shown_variables.is_none()
            && rule
                .hidden_mixins_and_functions
                .as_ref()
                .is_some_and(HashSet::is_empty)
            && rule
                .hidden_variables
                .as_ref()
                .is_some_and(HashSet::is_empty)
        {
            module
        } else {
            Arc::new(RefCell::new(Module::Forwarded(ForwardedModule::new(
                module, rule,
            ))))
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ModuleScope {
    pub variables: Arc<dyn MapView<Value = Value>>,
    pub mixins: Arc<dyn MapView<Value = Mixin>>,
    pub functions: Arc<dyn MapView<Value = SassFunction>>,
}

impl ModuleScope {
    pub fn new() -> Self {
        Self {
            variables: Arc::new(BaseMapView(Arc::new(RefCell::new(BTreeMap::new())))),
            mixins: Arc::new(BaseMapView(Arc::new(RefCell::new(BTreeMap::new())))),
            functions: Arc::new(BaseMapView(Arc::new(RefCell::new(BTreeMap::new())))),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Module {
    Environment {
        scope: ModuleScope,
        /// The modules this module loaded, in load order. Walked at the end
        /// of compilation so a downstream module's extensions apply to the
        /// modules it loaded and no further.
        upstream: Vec<Arc<RefCell<Module>>>,
        /// The selectors this module's CSS registered and the extensions its
        /// `@extend` rules declared.
        extension_store: ExtensionStore,
        env: Environment,
        /// The URL the module was loaded from, as the module cache keys it.
        /// `None` for the dummy module an `@import` builds, which has no file
        /// of its own.
        url: Option<PathBuf>,
    },
    Builtin {
        scope: ModuleScope,
        /// The `sass:` URL, such as `sass:math`.
        url: PathBuf,
    },
    Forwarded(ForwardedModule),
    Shadowed(ShadowedModule),
}

/// The modules a stylesheet loaded by namespace.
///
/// Member access through a namespace, `ns.$var`, goes by [`Identifier`], so
/// `a_b.$x` and `a-b.$x` reach one module. A namespace given to a `meta`
/// function as a string is looked up as written, the way dart-sass does: it
/// is a value, not an identifier token, so `"a_b"` names a module only when
/// the `@use` spelt it that way.
#[derive(Debug, Clone)]
pub(crate) struct Modules {
    by_name: BTreeMap<Identifier, Arc<RefCell<Module>>>,
    /// The namespaces as the `@use` rules wrote them.
    as_written: BTreeSet<String>,
}

impl Modules {
    pub fn new() -> Self {
        Self {
            by_name: BTreeMap::new(),
            as_written: BTreeSet::new(),
        }
    }

    /// Registers `module` under the namespace `name`, as the `@use` rule wrote
    /// it. Errors when the namespace is taken.
    pub fn insert(
        &mut self,
        name: &str,
        module: Arc<RefCell<Module>>,
        span: Span,
    ) -> SassResult<()> {
        let ident = Identifier::from(name);

        if self.by_name.contains_key(&ident) {
            return Err((
                format!("There's already a module with namespace \"{}\".", ident),
                span,
            )
                .into());
        }

        self.by_name.insert(ident, module);
        self.as_written.insert(name.to_owned());

        Ok(())
    }

    pub fn get(&self, name: Identifier, span: Span) -> SassResult<Arc<RefCell<Module>>> {
        match self.by_name.get(&name) {
            Some(v) => Ok(Arc::clone(v)),
            None => Err(Self::no_module(name.as_str(), span)),
        }
    }

    /// Looks a namespace up by the exact string a `meta` function received.
    pub fn get_as_written(&self, name: &str, span: Span) -> SassResult<Arc<RefCell<Module>>> {
        if !self.as_written.contains(name) {
            return Err(Self::no_module(name, span));
        }

        self.get(Identifier::from(name), span)
    }

    pub fn get_mut(
        &mut self,
        name: Identifier,
        span: Span,
    ) -> SassResult<&mut Arc<RefCell<Module>>> {
        match self.by_name.get_mut(&name) {
            Some(v) => Ok(v),
            None => Err(Self::no_module(name.as_str(), span)),
        }
    }

    fn no_module(name: &str, span: Span) -> Box<crate::error::SassError> {
        (
            format!("There is no module with namespace \"{}\".", name),
            span,
        )
            .into()
    }
}

fn member_map<V: fmt::Debug + Clone + 'static>(
    local: Arc<dyn MapView<Value = V>>,
    others: Vec<Arc<dyn MapView<Value = V>>>,
) -> Arc<dyn MapView<Value = V>> {
    let local_map = PublicMemberMapView(local);

    if others.is_empty() {
        return Arc::new(local_map);
    }

    let mut all_maps: Vec<Arc<dyn MapView<Value = V>>> =
        others.into_iter().filter(|map| !map.is_empty()).collect();

    all_maps.push(Arc::new(local_map));

    // todo: potential optimization when all_maps.len() == 1
    Arc::new(MergedMapView::new(all_maps))
}

impl Module {
    /// Whether a `with (...)` clause naming `variables` could have configured
    /// this module when it was loaded: one of the names is declared
    /// `!default` at the module's root, or reaches such a declaration through
    /// the module's `@forward`s.
    ///
    /// This gates the "already loaded, so it can't be configured" error: a
    /// clause whose names could never have applied to the module is not a
    /// double configuration, so loading a forwarding file with a
    /// configuration meant for its own variables stays legal even when the
    /// file it forwards was loaded earlier.
    pub(crate) fn could_have_been_configured(&self, variables: &HashSet<Identifier>) -> bool {
        match self {
            Self::Builtin { .. } => false,
            Self::Environment { env, .. } => {
                if variables
                    .iter()
                    .any(|name| (*env.configurable_variables).borrow().contains(name))
                {
                    return true;
                }

                (*env.forwarded_modules)
                    .borrow()
                    .iter()
                    .any(|module| (**module).borrow().could_have_been_configured(variables))
            }
            Self::Forwarded(forwarded) => forwarded.could_have_been_configured(variables),
            Self::Shadowed(shadowed) => shadowed.could_have_been_configured(variables),
        }
    }

    pub fn new_env(
        env: Environment,
        extension_store: ExtensionStore,
        upstream: Vec<Arc<RefCell<Module>>>,
        url: Option<PathBuf>,
    ) -> Self {
        let variables = {
            let variables = (*env.forwarded_modules).borrow();
            let variables = variables
                .iter()
                .map(|module| Arc::clone(&(*module).borrow().scope().variables));
            let this = Arc::new(BaseMapView(env.global_vars()));
            member_map(this, variables.collect())
        };

        let mixins = {
            let mixins = (*env.forwarded_modules).borrow();
            let mixins = mixins
                .iter()
                .map(|module| Arc::clone(&(*module).borrow().scope().mixins));
            let this = Arc::new(BaseMapView(env.global_mixins()));
            member_map(this, mixins.collect())
        };

        let functions = {
            let functions = (*env.forwarded_modules).borrow();
            let functions = functions
                .iter()
                .map(|module| Arc::clone(&(*module).borrow().scope().functions));
            let this = Arc::new(BaseMapView(env.global_functions()));
            member_map(this, functions.collect())
        };

        let scope = ModuleScope {
            variables,
            mixins,
            functions,
        };

        Module::Environment {
            scope,
            upstream,
            extension_store,
            env,
            url,
        }
    }

    pub fn new_builtin(url: &str) -> Self {
        Module::Builtin {
            scope: ModuleScope::new(),
            url: PathBuf::from(url),
        }
    }

    /// The URL this module was loaded from: the module cache's key for a
    /// stylesheet, the `sass:` URL for a built-in module. A forwarded or
    /// shadowed view answers for the module it wraps. `None` for the dummy
    /// module an `@import` builds.
    pub(crate) fn url(&self) -> Option<PathBuf> {
        match self {
            Self::Environment { url, .. } => url.clone(),
            Self::Builtin { url, .. } => Some(url.clone()),
            Self::Forwarded(ForwardedModule { inner, .. })
            | Self::Shadowed(ShadowedModule { inner, .. }) => (**inner).borrow().url(),
        }
    }

    /// The namespace a `@use` of this module's URL would get without an `as`
    /// clause: the file's base name without a leading underscore or an
    /// extension, provided that is a valid identifier. `meta.inspect` prints
    /// it inside `get-module(...)`, and prints `get-module()` when there is
    /// none.
    pub(crate) fn default_namespace(&self) -> Option<String> {
        default_namespace(&self.url()?)
    }

    pub(crate) fn scope(&self) -> ModuleScope {
        match self {
            Self::Builtin { scope, .. }
            | Self::Environment { scope, .. }
            | Self::Forwarded(ForwardedModule { scope, .. })
            | Self::Shadowed(ShadowedModule { scope, .. }) => scope.clone(),
        }
    }

    pub fn get_var(&self, name: Spanned<Identifier>) -> SassResult<Value> {
        let scope = self.scope();

        match scope.variables.get(name.node) {
            Some(v) => Ok(v),
            None => Err(("Undefined variable.", name.span).into()),
        }
    }

    pub fn get_var_no_err(&self, name: Identifier) -> Option<Value> {
        let scope = self.scope();

        scope.variables.get(name)
    }

    pub fn get_mixin_no_err(&self, name: Identifier) -> Option<Mixin> {
        let scope = self.scope();

        scope.mixins.get(name)
    }

    pub fn update_var(&mut self, name: Spanned<Identifier>, value: Value) -> SassResult<()> {
        let scope = match self {
            Self::Builtin { .. } => {
                return Err(("Cannot modify built-in variable.", name.span).into());
            }
            Self::Environment { scope, .. }
            | Self::Forwarded(ForwardedModule { scope, .. })
            | Self::Shadowed(ShadowedModule { scope, .. }) => scope.clone(),
        };

        if scope.variables.insert(name.node, value).is_none() {
            return Err(("Undefined variable.", name.span).into());
        }

        Ok(())
    }

    pub fn get_mixin(&self, name: Spanned<Identifier>) -> SassResult<Mixin> {
        let scope = self.scope();

        match scope.mixins.get(name.node) {
            Some(v) => Ok(v),
            None => Err(("Undefined mixin.", name.span).into()),
        }
    }

    pub fn insert_builtin_mixin(
        &mut self,
        name: &'static str,
        mixin: BuiltinMixin,
        accepts_content: bool,
    ) {
        let scope = self.scope();
        let ident: Identifier = name.into();

        scope
            .mixins
            .insert(ident, Mixin::Builtin(mixin, ident, accepts_content));
    }

    /// The module's mixins as a map from name to first-class mixin, for
    /// `meta.module-mixins`.
    pub fn mixins(&self, span: Span) -> SassMap {
        SassMap::new_with(
            self.scope()
                .mixins
                .iter()
                .into_iter()
                .filter(|(key, _)| !key.as_str().starts_with('-'))
                .map(|(key, value)| {
                    (
                        Value::String(key.to_string(), QuoteKind::Quoted).span(span),
                        Value::MixinRef(SassMixin::new(value)),
                    )
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn insert_builtin_var(&mut self, name: &'static str, value: Value) {
        let ident = name.into();

        let scope = self.scope();

        scope.variables.insert(ident, value);
    }

    pub fn get_fn(&self, name: Identifier) -> Option<SassFunction> {
        let scope = self.scope();

        scope.functions.get(name)
    }

    pub fn var_exists(&self, name: Identifier) -> bool {
        let scope = self.scope();

        scope.variables.get(name).is_some()
    }

    pub fn mixin_exists(&self, name: Identifier) -> bool {
        let scope = self.scope();

        scope.mixins.get(name).is_some()
    }

    pub fn fn_exists(&self, name: Identifier) -> bool {
        let scope = self.scope();

        scope.functions.get(name).is_some()
    }

    pub fn insert_builtin(
        &mut self,
        name: &'static str,
        function: fn(ArgumentResult, &mut Visitor) -> SassResult<Value>,
    ) {
        let ident = name.into();

        let scope = match self {
            Self::Builtin { scope, .. } => scope,
            _ => unreachable!(),
        };

        scope
            .functions
            .insert(ident, SassFunction::Builtin(Builtin::new(function), ident));
    }

    pub fn functions(&self, span: Span) -> SassMap {
        SassMap::new_with(
            self.scope()
                .functions
                .iter()
                .into_iter()
                .filter(|(key, _)| !key.as_str().starts_with('-'))
                .map(|(key, value)| {
                    (
                        Value::String(key.to_string(), QuoteKind::Quoted).span(span),
                        Value::FunctionRef(Box::new(value)),
                    )
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn variables(&self, span: Span) -> SassMap {
        SassMap::new_with(
            self.scope()
                .variables
                .iter()
                .into_iter()
                .filter(|(key, _)| !key.as_str().starts_with('-'))
                .map(|(key, value)| {
                    (
                        Value::String(key.to_string(), QuoteKind::Quoted).span(span),
                        value,
                    )
                })
                .collect::<Vec<_>>(),
        )
    }
}

/// Gives each function registered in the builtin module `name` the parameter
/// lists dart-sass declares for it, so a call is checked against them.
///
/// Attaching them here, once the module is declared, keeps the lists in one
/// table rather than spread across every `insert_builtin` call.
fn with_signatures(module: Module, name: &str) -> Module {
    let functions = module.scope().functions;
    for (ident, function) in functions.iter() {
        if let SassFunction::Builtin(builtin, _) = function {
            let signatures = crate::builtin::signatures::module(name, ident.as_str());
            functions.insert(
                ident,
                SassFunction::Builtin(builtin.with_signatures(signatures), ident),
            );
        }
    }
    module
}

pub(crate) fn declare_module_color() -> Module {
    let mut module = Module::new_builtin("sass:color");
    color::declare(&mut module);
    with_signatures(module, "color")
}

pub(crate) fn declare_module_list() -> Module {
    let mut module = Module::new_builtin("sass:list");
    list::declare(&mut module);
    with_signatures(module, "list")
}

pub(crate) fn declare_module_map() -> Module {
    let mut module = Module::new_builtin("sass:map");
    map::declare(&mut module);
    with_signatures(module, "map")
}

pub(crate) fn declare_module_math() -> Module {
    let mut module = Module::new_builtin("sass:math");
    math::declare(&mut module);
    with_signatures(module, "math")
}

pub(crate) fn declare_module_meta() -> Module {
    let mut module = Module::new_builtin("sass:meta");
    meta::declare(&mut module);
    with_signatures(module, "meta")
}

pub(crate) fn declare_module_selector() -> Module {
    let mut module = Module::new_builtin("sass:selector");
    selector::declare(&mut module);
    with_signatures(module, "selector")
}

pub(crate) fn declare_module_string() -> Module {
    let mut module = Module::new_builtin("sass:string");
    string::declare(&mut module);
    with_signatures(module, "string")
}

/// The namespace `@use url;` would give a module, or `None` when the URL's
/// base name is not a valid identifier and the `@use` would need an `as`
/// clause.
///
/// This is dart-sass's `defaultNamespace`: the last path segment, without a
/// leading underscore and without anything from the first dot on. A `sass:`
/// URL has a single segment, the module name.
pub(crate) fn default_namespace(url: &Path) -> Option<String> {
    let url = url.to_string_lossy();
    let base_name = match url.strip_prefix("sass:") {
        Some(name) => name,
        None => Path::new(url.as_ref())
            .file_name()
            .map(|name| name.to_str().unwrap_or(""))
            .unwrap_or(""),
    };

    let start = usize::from(base_name.starts_with('_'));
    let end = base_name.find('.').unwrap_or(base_name.len());
    let namespace = &base_name[start..end];

    is_identifier(namespace).then(|| namespace.to_owned())
}

/// Whether `text` is one whole Sass identifier: an optional leading hyphen,
/// then either a second hyphen or a name-start character, then name
/// characters to the end. Escapes are not recognised; a file name containing
/// a backslash gets no default namespace.
fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars().peekable();

    if chars.peek() == Some(&'-') {
        chars.next();
        if chars.peek() == Some(&'-') {
            chars.next();
            return chars.all(is_name);
        }
    }

    match chars.next() {
        Some(c) if is_name_start(c) => chars.all(is_name),
        _ => false,
    }
}
