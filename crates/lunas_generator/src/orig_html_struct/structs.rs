use lunas_html_parser::{Dom as RawDom, Element as RawElm, Node as RawNode};
use std::collections::HashMap;

use crate::transformers::utils::UUID_GENERATOR;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub uuid: String,
    pub content: NodeContent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeContent {
    Element(Element),
    TextNode(String),
    Comment(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub tag_name: String,
    pub attributes: HashMap<String, Option<String>>,
    pub children: Vec<Node>,
}

impl Element {
    pub fn new_from_raw(raw_elm: RawElm, node_vec: Vec<Node>) -> Element {
        Element {
            attributes: raw_elm.attributes.clone(),
            children: node_vec,
            tag_name: raw_elm.name,
        }
    }

    pub fn remove_child(
        &mut self,
        child_uuid: &String,
        custom_component_names: &Vec<String>,
    ) -> (Node, u64, u64, Option<u64>) {
        let idx = self
            .children
            .iter()
            .position(|child| child.uuid == *child_uuid)
            .unwrap();
        let mut cur = idx.clone() + 1;
        let (distance, idx_of_ref) = loop {
            if cur >= self.children.len() {
                break (cur as u64 - idx as u64, None);
            }
            let cur_child = &self.children[cur];
            match &cur_child.content {
                NodeContent::Element(elm) => {
                    if !elm.attributes.contains_key("$$$conditional$$$")
                        && !custom_component_names.contains(&elm.tag_name)
                    {
                        break (cur as u64 - idx as u64, Some(cur as u64));
                    }
                }
                _ => {}
            }
            cur += 1;
        };
        let elm_node = self.children[idx].clone();
        self.children.retain(|child| child.uuid != *child_uuid);
        (elm_node, idx as u64, distance, idx_of_ref)
    }

    pub fn attributes_without_meta(&self) -> HashMap<String, Option<String>> {
        let mut attributes = self.attributes.clone();
        attributes.remove("$$$conditional$$$");
        attributes
    }
}

impl Node {
    fn new_comment(comment: &String) -> Node {
        Node {
            uuid: UUID_GENERATOR.lock().unwrap().gen(),
            content: NodeContent::Comment(comment.clone()),
        }
    }

    fn new_text(text: &String) -> Node {
        Node {
            uuid: UUID_GENERATOR.lock().unwrap().gen(),
            content: NodeContent::TextNode(text.clone()),
        }
    }

    fn new_from_raw(elm: &RawElm) -> Node {
        let mut children = vec![];
        for child in &elm.children {
            children.push(Node::new_from_node(child));
        }
        Node {
            uuid: UUID_GENERATOR.lock().unwrap().gen(),
            content: NodeContent::Element(Element::new_from_raw(elm.clone(), children)),
        }
    }

    pub fn new_from_dom(raw_dom: &RawDom) -> Result<Node, String> {
        match raw_dom.children.len() {
            0 => Err("Root element has no child".to_string()),
            1 => Ok(Node::new_from_node(&raw_dom.children[0])),
            _ => Err("Root element has more than one child".to_string()),
        }
    }

    pub fn new_from_node(raw_node: &RawNode) -> Node {
        match raw_node {
            RawNode::Text(text) => Node::new_text(text),
            RawNode::Element(elm) => Node::new_from_raw(elm),
            RawNode::Comment(comment) => Node::new_comment(comment),
        }
    }
}

impl ToString for Node {
    fn to_string(&self) -> String {
        match &self.content {
            NodeContent::Element(elm) => elm.to_string(),
            NodeContent::TextNode(text) => text.clone(),
            NodeContent::Comment(comment) => comment.clone(),
        }
    }
}

impl ToString for Element {
    fn to_string(&self) -> String {
        let mut attribute_str = String::new();

        let mut attributes: Vec<_> = self.attributes.iter().collect();
        attributes.sort_by(|a, b| a.0.cmp(b.0));

        for (key, value) in attributes {
            if let Some(value) = value {
                attribute_str.push_str(&format!(" {}=\"{}\"", key, value));
            } else {
                attribute_str.push_str(&format!(" {}", key));
            }
        }

        let mut children = String::new();
        for child in &self.children {
            children.push_str(&child.to_string());
        }
        format!(
            "<{}{}>{}</{}>",
            self.tag_name, attribute_str, children, self.tag_name
        )
        // TODO: Make it a self-closing tag for certain tags
        // match self.children.len() == 0 {
        //     true => {
        //         format!("<{}{} />", self.tag_name, attribute_str)
        //     }
        //     false => {
        //         let mut children = String::new();
        //         for child in &self.children {
        //             children.push_str(&child.to_string());
        //         }
        //         format!(
        //             "<{}{}>{}</{}>",
        //             self.tag_name, attribute_str, children, self.tag_name
        //         )
        //     }
        // }
    }
}

mod tests {
    #[test]
    fn test_node_to_string() {
        let raw_html = "<div><p>hello</p></div>";
        let raw_node = lunas_html_parser::Dom::parse(raw_html).unwrap();
        let el = raw_node.children[0].clone();
        let node = crate::orig_html_struct::structs::Node::new_from_node(&el);
        assert_eq!(node.to_string(), raw_html);
    }
}
