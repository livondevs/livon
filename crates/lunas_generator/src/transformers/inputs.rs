use lunas_parser::PropsInput;
use num_bigint::BigUint;

use crate::structs::transform_info::VariableNameAndAssignedNumber;

pub fn generate_input_variable_decl(
    inputs: &Vec<&PropsInput>,
    variables: &mut Vec<VariableNameAndAssignedNumber>,
) -> Option<String> {
    for (i, input) in inputs.iter().enumerate() {
        variables.push(VariableNameAndAssignedNumber {
            name: input.variable_name.clone(),
            assignment: BigUint::from(2u32).pow(i as u32),
            to_add_value_accessor: true,
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
