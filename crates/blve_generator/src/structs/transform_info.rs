use crate::{orig_html_struct::structs::Node, transformers::utils::append_v_to_vars_in_html};

#[derive(Debug, Clone)]
pub enum TransformInfo {
    AddStringToPosition(AddStringToPosition),
    RemoveStatement(RemoveStatement),
    ReplaceText(ReplaceText),
}

#[derive(Debug, Clone)]
pub struct AddStringToPosition {
    pub position: u32,
    pub string: String,
}

#[derive(Debug, Clone)]
pub struct RemoveStatement {
    pub start_position: u32,
    pub end_position: u32,
}

#[derive(Debug, Clone)]
pub struct ReplaceText {
    pub start_position: u32,
    pub end_position: u32,
    pub string: String,
}

#[derive(Debug)]
pub struct VariableNameAndAssignedNumber {
    pub name: String,
    pub assignment: u32,
}

#[derive(Debug)]
pub struct ActionAndTarget {
    pub action_name: String,
    pub action: EventTarget,
    pub target: String,
    pub ctx: Vec<String>,
}

// FIXME: 命名
#[derive(Debug)]
pub struct NeededIdName {
    pub id_name: String,
    pub to_delete: bool,
    pub node_id: String,
    pub ctx: Vec<String>,
}

#[derive(Debug)]
pub enum EventTarget {
    RefToFunction(String),
    Statement(String),
    EventBindingStatement(EventBindingStatement),
}

#[derive(Debug)]
pub struct EventBindingStatement {
    pub statement: String,
    pub arg: String,
}

impl EventTarget {
    pub fn to_string(&self, variables: &Vec<String>) -> String {
        match self {
            EventTarget::RefToFunction(function_name) => function_name.clone(),
            EventTarget::Statement(statement) => {
                // cosider using returned variables
                let (statement, _) = append_v_to_vars_in_html(statement.as_str(), variables);
                format!("()=>{}", statement)
            }
            EventTarget::EventBindingStatement(statement) => {
                format!("({})=>{}", statement.arg, statement.statement)
            }
        }
    }
}

impl EventTarget {
    pub fn new(content: String, variables: &Vec<String>) -> Self {
        // FIXME: This is a hacky way to check if the content is a statement or a function
        if content.trim().ends_with(")") {
            EventTarget::Statement(content)
        } else if word_is_one_word(content.as_str()) {
            EventTarget::RefToFunction(content)
        } else {
            EventTarget::Statement(append_v_to_vars_in_html(content.as_str(), &variables).0)
        }
    }
}

fn word_is_one_word(word: &str) -> bool {
    word.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfBlockInfo {
    pub parent_id: String,
    pub target_if_blk_id: String,
    pub distance_to_next_elm: u64,
    pub target_anchor_id: Option<String>,
    pub node: Node,
    pub ref_text_node_id: Option<String>,
    pub condition: String,
    pub condition_dep_vars: Vec<String>,
    pub ctx_under_if: Vec<String>,
    pub ctx_over_if: Vec<String>,
    pub if_blk_id: String,
    pub element_location: Vec<usize>,
}

impl IfBlockInfo {
    pub fn generate_ctx_num(&self, if_blocks_infos: &Vec<IfBlockInfo>) -> usize {
        let mut ctx_num: u64 = 0;
        for (index, if_blk) in if_blocks_infos.iter().enumerate() {
            if self.ctx_over_if.contains(&if_blk.target_if_blk_id) {
                let blk_num: u64 = (2 as u64).pow(index as u32);
                ctx_num = ctx_num | blk_num;
            }
        }

        ctx_num as usize
    }

    pub fn find_children(&self, if_blocks_infos: &Vec<IfBlockInfo>) -> Vec<IfBlockInfo> {
        let mut children: Vec<IfBlockInfo> = vec![];
        for if_blk in if_blocks_infos {
            if if_blk.ctx_under_if.starts_with(&self.ctx_under_if)
                && if_blk.ctx_under_if.len() == self.ctx_under_if.len() + 1
            {
                children.push(if_blk.clone());
            }
        }

        children
    }
}

pub fn sort_if_blocks(if_blocks: &mut Vec<IfBlockInfo>) {
    if_blocks.sort_by(|a, b| a.element_location.cmp(&b.element_location));
}

#[derive(Debug, Clone)]
pub struct CustomComponentBlockInfo {
    pub parent_id: String,
    pub target_if_blk_id: String,
    pub distance_to_next_elm: u64,
    pub have_sibling_elm: bool,
    pub target_anchor_id: Option<String>,
    pub component_name: String,
    pub ref_text_node_id: Option<String>,
    pub ctx: Vec<String>,
    pub custom_component_block_id: String,
    pub element_location: Vec<usize>,
    pub is_routing_component: bool,
}

#[derive(Debug, Clone)]
pub struct ManualRendererForTextNode {
    pub parent_id: String,
    pub text_node_id: String,
    pub distance_to_next_elm: u64,
    pub dep_vars: Vec<String>,
    pub content: String,
    pub ctx: Vec<String>,
    pub element_location: Vec<usize>,
    pub target_anchor_id: Option<String>,
}

pub enum TextNodeRenderer {
    ManualRenderer(ManualRendererForTextNode),
    IfBlockRenderer(IfBlockInfo),
    CustomComponentRenderer(CustomComponentBlockInfo),
}

impl TextNodeRenderer {
    pub fn get_element_location(&self) -> &Vec<usize> {
        match self {
            TextNodeRenderer::ManualRenderer(renderer) => &renderer.element_location,
            TextNodeRenderer::IfBlockRenderer(renderer) => &renderer.element_location,
            TextNodeRenderer::CustomComponentRenderer(renderer) => &renderer.element_location,
        }
    }
}

pub struct TextNodeRendererGroup {
    pub renderers: Vec<TextNodeRenderer>,
}

impl TextNodeRendererGroup {
    pub fn sort_by_rendering_order(&mut self) {
        self.renderers.sort_by(|a, b| {
            return a.get_element_location().cmp(&b.get_element_location());
        });
    }

    pub fn new(
        if_blk: &Vec<IfBlockInfo>,
        text_node_renderer: &Vec<ManualRendererForTextNode>,
        custom_component_block: &Vec<CustomComponentBlockInfo>,
    ) -> Self {
        let mut renderers: Vec<TextNodeRenderer> = vec![];
        for if_blk in if_blk {
            renderers.push(TextNodeRenderer::IfBlockRenderer(if_blk.clone()));
        }
        for txt_node_renderer in text_node_renderer {
            renderers.push(TextNodeRenderer::ManualRenderer(txt_node_renderer.clone()));
        }
        for custom_component_block in custom_component_block {
            renderers.push(TextNodeRenderer::CustomComponentRenderer(
                custom_component_block.clone(),
            ));
        }

        let mut render_grp = TextNodeRendererGroup { renderers };
        render_grp.sort_by_rendering_order();
        render_grp
    }
}
