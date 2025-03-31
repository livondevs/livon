use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HtmlManipulator {
    pub target_uuid: String,
    pub manipulations: HtmlManipulation,
}

#[derive(Debug, Clone)]
pub enum HtmlManipulation {
    RemoveChildForIfStatement(RemoveChildForIfStatement),
    RemoveChildForForStatement(RemoveChildForRepeatStatement),
    RemoveChildForCustomComponent(RemoveChildForCustomComponent),
    SetIdForReactiveContent(SetIdToParentForChildReactiveText),
    RemoveChildTextNode(RemoveChildTextNode),
}

#[derive(Debug, Clone)]
pub struct RemoveChildForIfStatement {
    // FIXME: child_uuid is exactly the same as block_id
    pub child_uuid: String,
    pub condition: String,
    pub original_condition: Option<String>,
    pub block_id: String,
    // TODO:ctxとlocをHtmlManipulatorに入れるか検討する
    pub ctx_under_if: Vec<String>,
    pub ctx_over_if: Vec<String>,
    pub elm_loc: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct RemoveChildForRepeatStatement {
    pub child_uuid: String,
    pub item_name: String,
    pub item_index: String,
    pub item_collection: String,
    pub block_id: String,
    pub ctx_under_for: Vec<String>,
    pub ctx_over_for: Vec<String>,
    pub elm_loc: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct RemoveChildForCustomComponent {
    pub component_name: String,
    pub attributes: HashMap<String, Option<String>>,
    pub child_uuid: String,
    pub ctx: Vec<String>,
    pub elm_loc: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct SetIdToParentForChildReactiveText {
    pub text: String,
    pub depenent_vars: Vec<String>,
    pub ctx: Vec<String>,
    pub elm_loc: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct RemoveChildTextNode {
    pub depenent_vars: Vec<String>,
    pub ctx: Vec<String>,
    pub elm_loc: Vec<usize>,
    pub child_uuid: String,
    pub content: String,
}
