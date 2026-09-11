use crate::builtin::builtin_imports::*;

/// map.get($map, $key, $keys...)
/// map-get($map, $key, $keys...)
///
/// If $keys is empty, returns the value in $map associated with $key.
/// If $map doesn’t have a value associated with $key, returns null.
/// If $keys is not empty, follows the set of keys including $key and
/// excluding the last key in $keys, from left to right, to find the
/// nested map targeted for searching.
/// Returns the value in the targeted map associated with the last key
/// in $keys.
/// Returns null if the map does not have a value associated with the
/// key, or if any key in $keys is missing from a map or references a
/// value that is not a map.
///
/// https://sass-lang.com/documentation/modules/map/
pub(crate) fn map_get(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    let key = args.get_err(1, "key")?;
    let map = args
        .get_err(0, "map")?
        .assert_map_with_name("map", args.span())?;

    // since we already extracted the map and first key,
    // neither will be returned in the variadic args list
    let keys = args.get_variadic()?;

    let mut val = map.get(&key).unwrap_or(Value::Null);
    for key in keys {
        // if at any point we find a value that's not a map,
        // we return null
        let val_map = match val.try_map() {
            Some(val_map) => val_map,
            None => return Ok(Value::Null),
        };

        val = val_map.get(&key).unwrap_or(Value::Null);
    }

    Ok(val)
}

pub(crate) fn map_has_key(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    let key = args.get_err(1, "key")?;
    let map = args
        .get_err(0, "map")?
        .assert_map_with_name("map", args.span())?;

    // since we already extracted the map and first key,
    // neither will be returned in the variadic args list
    let keys = args.get_variadic()?;

    let mut val = match map.get(&key) {
        Some(v) => v,
        None => return Ok(Value::False),
    };
    for key in keys {
        // if at any point we find a value that's not a map,
        // we return null
        let val_map = match val.try_map() {
            Some(val_map) => val_map,
            None => return Ok(Value::False),
        };

        val = match val_map.get(&key) {
            Some(v) => v,
            None => return Ok(Value::False),
        };
    }

    Ok(Value::True)
}

pub(crate) fn map_keys(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let map = args
        .get_err(0, "map")?
        .assert_map_with_name("map", args.span())?;
    Ok(Value::List(
        map.keys(),
        ListSeparator::Comma,
        Brackets::None,
    ))
}

pub(crate) fn map_values(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let map = args
        .get_err(0, "map")?
        .assert_map_with_name("map", args.span())?;
    Ok(Value::List(
        map.values(),
        ListSeparator::Comma,
        Brackets::None,
    ))
}

/// `map.merge` in dart-sass's two overloads: `$map1, $map2` merges two maps,
/// and `$map1, $args...` merges its last argument into the map found at the
/// key path the others name.
///
/// The overload comes from the call's parameter lists. Counting arguments
/// cannot tell `map.merge($m, $k, $map2: $n)` -- the `$args...` form with a
/// stray named argument -- from the two-map form.
pub(crate) fn map_merge(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    let span = args.span();

    let mut map1 = args
        .get_err(0, "map1")?
        .assert_map_with_name("map1", span)?;

    let (keys, map2) = if args.overload == 0 {
        let map2 = args
            .get_err(1, "map2")?
            .assert_map_with_name("map2", span)?;
        (Vec::new(), map2)
    } else {
        let mut rest: Vec<Value> = args.positional.drain(1..).collect();
        let Some(last) = rest.pop() else {
            return Err(("Expected $args to contain a key.", span).into());
        };
        if rest.is_empty() {
            return Err(("Expected $args to contain a map.", span).into());
        }
        let map2 = last.assert_map_with_name("map2", span)?;
        let keys: Vec<Spanned<Value>> = rest
            .into_iter()
            .map(|node| Spanned { node, span })
            .collect();
        (keys, map2)
    };

    if keys.is_empty() {
        map1.merge(map2);
    } else {
        let mut current_map = map1.clone();
        let mut map_queue = Vec::new();

        for key in keys {
            match current_map.get(&key) {
                Some(Value::Map(m1)) => {
                    current_map = m1.clone();
                    map_queue.push((key, m1));
                }
                Some(..) | None => {
                    current_map = SassMap::new();
                    map_queue.push((key, SassMap::new()));
                }
            }
        }

        match map_queue.last_mut() {
            Some((_, m)) => {
                m.merge(map2);
            }
            None => unreachable!(),
        };

        while let Some((key, queued_map)) = map_queue.pop() {
            match map_queue.last_mut() {
                Some((_, map)) => {
                    map.insert(key, Value::Map(queued_map));
                }
                None => {
                    map1.insert(key, Value::Map(queued_map));
                    break;
                }
            }
        }
    }

    args.assert_named_consumed()?;

    Ok(Value::Map(map1))
}

pub(crate) fn map_remove(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    let mut map = args
        .get_err(0, "map")?
        .assert_map_with_name("map", args.span())?;
    let keys = args.get_variadic()?;
    for key in keys {
        map.remove(&key);
    }
    Ok(Value::Map(map))
}

/// `map.set` in dart-sass's two overloads: `$map, $key, $value` sets one key,
/// and `$map, $args...` sets its last argument at the key path the others
/// name.
///
/// The overload comes from the call's parameter lists, for the reason
/// [`map_merge`] gives.
pub(crate) fn map_set(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    let span = args.span();

    let mut map = args.get_err(0, "map")?.assert_map_with_name("map", span)?;

    let (keys, key, value) = if args.overload == 0 {
        let key = args.get_err(1, "key")?;
        let value = args.get_err(2, "value")?;
        (Vec::new(), key, value)
    } else {
        let mut rest: Vec<Value> = args.positional.drain(1..).collect();
        let Some(value) = rest.pop() else {
            return Err(("Expected $args to contain a key.", span).into());
        };
        let Some(key) = rest.pop() else {
            return Err(("Expected $args to contain a value.", span).into());
        };
        let keys: Vec<Spanned<Value>> = rest
            .into_iter()
            .map(|node| Spanned { node, span })
            .collect();
        (keys, key, value)
    };
    let key = Spanned { node: key, span };

    if keys.is_empty() {
        map.insert(key, value);
    } else {
        let mut current_map = map.clone();
        let mut map_queue = Vec::new();

        for key in keys {
            match current_map.get(&key) {
                Some(Value::Map(m1)) => {
                    current_map = m1.clone();
                    map_queue.push((key, m1));
                }
                Some(..) | None => {
                    current_map = SassMap::new();
                    map_queue.push((key, SassMap::new()));
                }
            }
        }

        match map_queue.last_mut() {
            Some((_, m)) => m.insert(key, value),
            None => unreachable!(),
        };

        while let Some((key, queued_map)) = map_queue.pop() {
            match map_queue.last_mut() {
                Some((_, next_map)) => {
                    next_map.insert(key, Value::Map(queued_map));
                }
                None => {
                    map.insert(key, Value::Map(queued_map));
                    break;
                }
            }
        }
    }

    args.assert_named_consumed()?;

    Ok(Value::Map(map))
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("map-get", Builtin::new(map_get));
    f.insert("map-has-key", Builtin::new(map_has_key));
    f.insert("map-keys", Builtin::new(map_keys));
    f.insert("map-values", Builtin::new(map_values));
    f.insert("map-merge", Builtin::new(map_merge));
    f.insert("map-remove", Builtin::new(map_remove));
}
