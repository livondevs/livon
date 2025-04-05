use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsFunctionDeps {
    pub name: String,
    pub depending_vars: HashSet<String>,
    pub depending_funcs: HashSet<String>,
}

pub trait Tidy {
    fn tidy(&mut self) -> ();
}

impl Tidy for Vec<JsFunctionDeps> {
    fn tidy(&mut self) -> () {
        let all_dependencies: Vec<(String, Vec<String>)> = self
            .iter()
            .map(|func| {
                (
                    func.name.clone(),
                    func.depending_vars.iter().cloned().collect::<Vec<String>>(),
                )
            })
            .collect();

        for func in self.iter_mut() {
            let all_depending_vars = func
                .depending_funcs
                .iter()
                .filter_map(|func_name| {
                    all_dependencies
                        .iter()
                        .find(|(name, _)| name == func_name)
                        .map(|(_, vars)| vars.clone())
                })
                .flatten()
                .collect::<Vec<String>>();
            func.depending_vars.extend(all_depending_vars);
            func.depending_funcs.clear();
        }
    }
}
