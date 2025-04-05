use lunas_parser::PropsInput;

use crate::structs::transform_info::VariableNameAndAssignedNumber;

pub fn generate_input_variable_decl(
    inputs: &Vec<&PropsInput>,
    variables: &mut Vec<VariableNameAndAssignedNumber>,
) -> Option<String> {
    for (i, input) in inputs.iter().enumerate() {
        variables.push(VariableNameAndAssignedNumber {
            name: input.variable_name.clone(),
            assignment: 2u128.pow(i as u32),
        });
    }
    let prop_name = inputs
        .iter()
        .map(|i| i.variable_name.clone())
        .collect::<Vec<String>>()
        .join(", ");
    match inputs.len() == 0 {
        true => return None,
        false => Some(format!("const {{ {} }} = args;", prop_name)),
    }
}
