use indexmap::IndexMap;
use serde_bridge::Value;
use std::hash::Hash;

fn merge_map<K: Hash + Eq>(mut l: IndexMap<K, Value>, r: IndexMap<K, Value>) -> IndexMap<K, Value> {
    for (k, rv) in r {
        match l.remove(&k) {
            Some(lv) => {
                let v = match (is_default(&lv), is_default(&rv)) {
                    (false, false) => merge(lv, rv),
                    (false, true) => lv,
                    (true, _) => rv,
                };

                l.insert(k, v);
            }
            None => {
                l.insert(k, rv);
            }
        };
    }
    l
}

pub fn merge(l: Value, r: Value) -> Value {
    use Value::*;

    match (l, r) {
        (Map(l), Map(r)) => Value::Map(merge_map(l, r)),
        (Struct(ln, lv), Struct(rn, rv)) if ln == rn => Value::Struct(ln, merge_map(lv, rv)),
        (
            StructVariant {
                name: ln,
                variant_index: lvi,
                variant: lv,
                fields: lf,
            },
            StructVariant {
                name: rn,
                variant_index: rvi,
                variant: rv,
                fields: rf,
            },
        ) if ln == rn && lvi == rvi && lv == rv => Value::StructVariant {
            name: ln,
            variant_index: lvi,
            variant: lv,
            fields: merge_map(lf, rf),
        },
        // Return `other` value if they are not merge-able
        (_, r) => r,
    }
}

pub fn is_default(value: &Value) -> bool {
    match value {
        Value::Bool(v) => !(*v),
        Value::I8(v) => *v == 0,
        Value::I16(v) => *v == 0,
        Value::I32(v) => *v == 0,
        Value::I64(v) => *v == 0,
        Value::I128(v) => *v == 0,
        Value::U8(v) => *v == 0,
        Value::U16(v) => *v == 0,
        Value::U32(v) => *v == 0,
        Value::U64(v) => *v == 0,
        Value::U128(v) => *v == 0,
        Value::F32(v) => *v == 0.0,
        Value::F64(v) => *v == 0.0,
        Value::Char(v) => *v == '\0',
        Value::Str(v) => v.is_empty(),
        Value::Bytes(v) => v.is_empty(),
        Value::None => true,
        Value::Some(v) => is_default(v),
        Value::Unit => true,
        Value::UnitStruct(_) => true,
        // We don't know which variant is default, always returns false instead.
        Value::UnitVariant { .. } => false,
        Value::NewtypeStruct(_, v) => is_default(v),
        Value::NewtypeVariant { value, .. } => is_default(value),
        Value::Seq(v) => v.is_empty(),
        Value::Tuple(v) => v.is_empty(),
        Value::TupleStruct(_, v) => v.is_empty(),
        Value::TupleVariant { fields, .. } => fields.is_empty(),
        Value::Map(v) => v.is_empty(),
        Value::Struct(_, v) => v.is_empty(),
        Value::StructVariant { fields, .. } => fields.is_empty(),
    }
}

fn merge_map_defaultable<K: Hash + Eq>(
    default: IndexMap<K, Value>,
    mut l: IndexMap<K, Value>,
    r: IndexMap<K, Value>,
) -> IndexMap<K, Value> {
    for (k, rv) in r {
        // Take unit as default if key not found.
        let dv = default.get(&k).unwrap_or(&Value::Unit);

        match l.remove(&k) {
            Some(lv) => {
                let v = match (&lv == dv, &rv == dv) {
                    (false, false) => merge(lv, rv),
                    (false, true) => lv,
                    (true, _) => rv,
                };

                l.insert(k, v);
            }
            None => {
                l.insert(k, rv);
            }
        };
    }
    l
}

pub fn merge_defaultable(default: Value, l: Value, r: Value) -> Value {
    use Value::*;

    match (default, l, r) {
        (Map(d), Map(l), Map(r)) => Value::Map(merge_map_defaultable(d, l, r)),
        (Struct(dn, dv), Struct(ln, lv), Struct(rn, rv)) if ln == rn && ln == dn => {
            Value::Struct(ln, merge_map_defaultable(dv, lv, rv))
        }
        (
            StructVariant {
                name: dn,
                variant_index: dvi,
                variant: dv,
                fields: df,
            },
            StructVariant {
                name: ln,
                variant_index: lvi,
                variant: lv,
                fields: lf,
            },
            StructVariant {
                name: rn,
                variant_index: rvi,
                variant: rv,
                fields: rf,
            },
        ) if ln == rn && lvi == rvi && lv == rv && ln == dn && lvi == dvi && lv == dv => {
            Value::StructVariant {
                name: ln,
                variant_index: lvi,
                variant: lv,
                fields: merge_map_defaultable(df, lf, rf),
            }
        }
        // Return `other` value if they are not merge-able
        (_, _, r) => r,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::indexmap;
    use serde_bridge::Value;
    use Value::*;

    #[test]
    fn test_merge() {
        let l = Map(indexmap! {
            Str("only_in_l".to_string()) => I64(1),
            Str("struct".to_string()) => Struct("test", indexmap! {
                "only_in_l" => U64(2),
                "common" => F64(3.4),
            })
        });
        let r = Map(indexmap! {
            Str("only_in_r".to_string()) => I64(2),
            Str("struct".to_string()) => Struct("test", indexmap! {
                "only_in_r" => U64(1),
                "common" => F64(5.6),
            })
        });
        let expect = Map(indexmap! {
            Str("only_in_l".to_string()) => I64(1),
            Str("only_in_r".to_string()) => I64(2),
            Str("struct".to_string()) => Struct("test", indexmap! {
                "only_in_l" => U64(2),
                "only_in_r" => U64(1),
                "common" => F64(5.6),
            })
        });
        assert_eq!(merge(l, r), expect)
    }

    #[test]
    fn test_merge_defaultable() {
        let default = Map(indexmap! {
            Str("default_value".to_string()) => I64(1),
            Str("struct".to_string()) => Struct("test", indexmap! {
                "only_in_l" => U64(100),
                "common" => F64(9.7),
            })
        });

        let l = Map(indexmap! {
            Str("only_in_l".to_string()) => I64(1),
            Str("struct".to_string()) => Struct("test", indexmap! {
                "only_in_l" => U64(2),
                "common" => F64(3.4),
            })
        });
        let r = Map(indexmap! {
            Str("only_in_r".to_string()) => I64(2),
            Str("struct".to_string()) => Struct("test", indexmap! {
                "only_in_r" => U64(1),
                "common" => F64(5.6),
            })
        });
        let expect = Map(indexmap! {
            Str("only_in_l".to_string()) => I64(1),
            Str("only_in_r".to_string()) => I64(2),
            Str("struct".to_string()) => Struct("test", indexmap! {
                "only_in_l" => U64(2),
                "only_in_r" => U64(1),
                "common" => F64(5.6),
            })
        });
        assert_eq!(merge_defaultable(default, l, r), expect)
    }
}
