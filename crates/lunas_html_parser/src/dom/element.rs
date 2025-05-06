use super::node::Node;
use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::default::Default;
use std::result::Result;

/// Normal: `<div></div>` or Void: `<meta/>`and `<meta>`
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
// TODO: Align with: https://html.spec.whatwg.org/multipage/syntax.html#elements-2
pub enum ElementVariant {
    /// A normal element can have children, ex: <div></div>.
    Normal,
    /// A void element can't have children, ex: <meta /> and <meta>
    Void,
}

pub type Attributes = HashMap<String, Option<String>>;

/// Most of the parsed html nodes are elements, except for text
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    /// The name / tag of the element
    pub name: String,

    /// The element variant, if it is of type void or not
    pub variant: ElementVariant,

    /// All of the elements attributes, except id and class
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(serialize_with = "ordered_map")]
    pub attributes: Attributes,

    /// All of the elements child nodes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Node>,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            variant: ElementVariant::Void,
            attributes: HashMap::new(),
            children: vec![],
        }
    }
}

impl ToString for Element {
    fn to_string(&self) -> String {
        let mut string = String::new();
        string.push_str(&format!("<{}", self.name));

        // self.attributesをソートしてから出力する
        let mut attributes: Vec<_> = self.attributes.iter().collect();
        attributes.sort_by(|a, b| a.0.cmp(b.0));

        for (key, value) in attributes {
            if let Some(value) = value {
                string.push_str(&format!(" {}=\"{}\"", key, value));
            } else {
                string.push_str(&format!(" {}", key));
            }
        }

        match self.variant {
            ElementVariant::Normal => {
                string.push_str(">");
                for child in &self.children {
                    string.push_str(&child.to_string());
                }
                string.push_str(&format!("</{}>", self.name));
            }
            ElementVariant::Void => {
                string.push_str("/>");
            }
        }

        string
    }
}

fn ordered_map<S: Serializer>(value: &Attributes, serializer: S) -> Result<S::Ok, S::Error> {
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
