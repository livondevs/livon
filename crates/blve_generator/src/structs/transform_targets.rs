#[derive(Debug)]
pub enum ElmAndReactiveInfo {
    ElmAndVariableRelation(ElmAndVariableContentRelation),
    ElmAndReactiveAttributeRelation(ElmAndReactiveAttributeRelation),
}

#[derive(Debug)]
pub struct ElmAndVariableContentRelation {
    pub elm_id: String,
    pub variable_names: Vec<String>,
    pub content_of_element: String,
    pub ctx_num: u64,
}

#[derive(Debug)]
pub struct ElmAndReactiveAttributeRelation {
    pub elm_id: String,
    pub reactive_attr: Vec<ReactiveAttr>,
    pub ctx_num: u64,
}

#[derive(Debug)]
pub struct ReactiveAttr {
    pub attribute_key: String,
    pub content_of_attr: String,
    pub variable_names: Vec<String>,
}
