#[derive(Debug, Clone)]
pub struct ContextCategories {
    pub if_ctx: Vec<String>,
    pub for_ctx: Vec<String>,
}

impl ContextCategories {
    pub fn is_under_for_clause(&self, ctx: &Vec<String>) -> bool {
        // TODO: 緊急 2重Forに対応する
        ctx.iter().any(|ctx_name| self.for_ctx.contains(&ctx_name.to_string()))
    }
}
