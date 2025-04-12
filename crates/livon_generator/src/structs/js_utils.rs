use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsSearchParent<'a> {
    NoneValue,
    MapValue(&'a Map<String, Value>),
    ParentIsArray,
}

impl<'a> JsSearchParent<'a> {
    pub fn is_some(&self) -> bool {
        match self {
            JsSearchParent::MapValue(_) => true,
            _ => false,
        }
    }

    pub fn unwrap(self) -> &'a Map<String, Value> {
        match self {
            JsSearchParent::MapValue(map) => map,
            _ => panic!("Called unwrap on a NoneValue or a ParentIsArray"),
        }
    }
}
