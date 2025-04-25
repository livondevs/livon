use lunas_html_parser::Dom;
use serde_json::Value;

#[derive(Debug)]
pub struct DetailedLanguageBlocks {
    pub dom: Dom,
    pub css: Option<String>,
    pub js: Option<JsBlock>,
}

#[derive(Debug)]
pub struct JsBlock {
    pub ast: Value,
    pub raw: String,
}
