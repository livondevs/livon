// TODO: リネームする
// TODO: 2つの共通のフィールドを持つ構造体を作る
#[derive(Debug)]
pub enum NodeAndReactiveInfo {
    ElmAndVariableRelation(ElmAndVariableContentRelation),
    ElmAndReactiveAttributeRelation(ElmAndReactiveAttributeRelation),
    TextAndVariableContentRelation(TextAndVariableContentRelation),
}

#[derive(Debug, Clone)]
pub struct ElmAndVariableContentRelation {
    pub elm_id: String,
    pub dep_vars: Vec<String>,
    pub content_of_element: String,
    pub elm_loc: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct TextAndVariableContentRelation {
    pub text_node_id: String,
    pub dep_vars: Vec<String>,
    pub content_of_element: String,
    pub elm_loc: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct ElmAndReactiveAttributeRelation {
    pub elm_id: String,
    pub reactive_attr: Vec<ReactiveAttr>,
    pub elm_loc: Vec<usize>,
}

pub fn sort_elm_and_reactive_info(_self: &mut Vec<NodeAndReactiveInfo>) {
    _self.sort_by(|a, b| {
        let a_elm_loc = match a {
            NodeAndReactiveInfo::ElmAndVariableRelation(elm_and_var) => elm_and_var.elm_loc.clone(),
            NodeAndReactiveInfo::ElmAndReactiveAttributeRelation(elm_and_reactive_attr) => {
                elm_and_reactive_attr.elm_loc.clone()
            }
            NodeAndReactiveInfo::TextAndVariableContentRelation(text_and_var) => {
                text_and_var.elm_loc.clone()
            }
        };
        let b_elm_loc = match b {
            NodeAndReactiveInfo::ElmAndVariableRelation(elm_and_var) => elm_and_var.elm_loc.clone(),
            NodeAndReactiveInfo::ElmAndReactiveAttributeRelation(elm_and_reactive_attr) => {
                elm_and_reactive_attr.elm_loc.clone()
            }
            NodeAndReactiveInfo::TextAndVariableContentRelation(text_and_var) => {
                text_and_var.elm_loc.clone()
            }
        };
        a_elm_loc.cmp(&b_elm_loc)
    });
}

#[derive(Debug, Clone)]
pub struct ReactiveAttr {
    pub attribute_key: String,
    pub content_of_attr: String,
    pub variable_names: Vec<String>,
}
