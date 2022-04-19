use indexmap::IndexMap;
use serde_bridge::Value;
use std::hash::Hash;

fn merge_map<K: Hash + Eq>(mut l: IndexMap<K, Value>, r: IndexMap<K, Value>) -> IndexMap<K, Value> {
    for (k, rv) in r {
        match l.remove(&k) {
            Some(lv) => {
                l.insert(k, merge(lv, rv));
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
}
