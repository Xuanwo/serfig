use indexmap::IndexMap;
use serde_bridge::Value;
use std::hash::Hash;

fn merge_map_with_default<K: Hash + Eq>(
    mut d: IndexMap<K, Value>,
    r: IndexMap<K, Value>,
) -> IndexMap<K, Value> {
    for (k, rv) in r {
        match d.remove(&k) {
            Some(lv) => {
                d.insert(k, merge_with_default(lv, rv));
            }
            None => {
                d.insert(k, rv);
            }
        };
    }
    d
}

pub fn merge_with_default(d: Value, r: Value) -> Value {
    use Value::*;

    match (d, r) {
        (Map(l), Map(r)) => Value::Map(merge_map_with_default(l, r)),
        (Struct(ln, lv), Struct(rn, rv)) if ln == rn => {
            Value::Struct(ln, merge_map_with_default(lv, rv))
        }
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
            fields: merge_map_with_default(lf, rf),
        },
        // Return `other` value if they are not merge-able
        (_, r) => r,
    }
}

fn merge_map<K: Hash + Eq>(
    mut d: IndexMap<K, Value>,
    mut l: IndexMap<K, Value>,
    r: IndexMap<K, Value>,
) -> IndexMap<K, Value> {
    for (k, rv) in r {
        let dv = d.remove(&k).expect("default must contain key");

        match l.remove(&k) {
            Some(lv) => {
                let v = match (dv == lv, dv == rv) {
                    (true, false) => rv,
                    (true, true) => dv,
                    (false, true) => lv,
                    (false, false) => merge(dv, lv, rv),
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

pub fn merge(d: Value, l: Value, r: Value) -> Value {
    use Value::*;

    match (d, l, r) {
        (Map(d), Map(l), Map(r)) => Value::Map(merge_map(d, l, r)),
        (Struct(dn, dv), Struct(ln, lv), Struct(rn, rv)) if dn == ln && ln == rn => {
            Value::Struct(ln, merge_map(dv, lv, rv))
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
                fields: merge_map(df, lf, rf),
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
        let d = Map(indexmap! {
            Str("only_in_l".to_string()) => I64(0),
            Str("only_in_r".to_string()) => I64(0),
            Str("struct".to_string()) => Struct("test", indexmap! {
                "only_in_l" => U64(0),
                "only_in_r" => U64(0),
                "common" => F64(0.0),
                "default_true" => Bool(true),
            })
        });
        let l = Map(indexmap! {
            Str("only_in_l".to_string()) => I64(1),
            Str("struct".to_string()) => Struct("test", indexmap! {
                "only_in_l" => U64(2),
                "common" => F64(3.4),
                "default_true" => Bool(true),
            })
        });
        let r = Map(indexmap! {
            Str("only_in_r".to_string()) => I64(2),
            Str("struct".to_string()) => Struct("test", indexmap! {
                "only_in_r" => U64(1),
                "common" => F64(5.6),
                "default_true" => Bool(false),
            })
        });
        let expect = Map(indexmap! {
            Str("only_in_l".to_string()) => I64(1),
            Str("only_in_r".to_string()) => I64(2),
            Str("struct".to_string()) => Struct("test", indexmap! {
                "only_in_l" => U64(2),
                "only_in_r" => U64(1),
                "common" => F64(5.6),
                "default_true" => Bool(false),
            })
        });

        assert_eq!(merge(d, l, r), expect)
    }
}
