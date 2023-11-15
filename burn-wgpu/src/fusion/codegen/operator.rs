use super::Variable;
use std::fmt::Display;

/// All operators that can be fused in a WGSL compute shader.
#[derive(Debug, Hash, Clone)]
pub enum Operator {
    Add {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Sub {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Mul {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Div {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Abs {
        input: Variable,
        out: Variable,
    },
    Exp {
        input: Variable,
        out: Variable,
    },
    Log {
        input: Variable,
        out: Variable,
    },
    Log1p {
        input: Variable,
        out: Variable,
    },
    Cos {
        input: Variable,
        out: Variable,
    },
    Sin {
        input: Variable,
        out: Variable,
    },
    Tanh {
        input: Variable,
        out: Variable,
    },
    Powf {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Erf {
        input: Variable,
        out: Variable,
    },
    AssignGlobal {
        input: Variable,
        out: Variable,
    },
    ReadGlobal {
        variable: Variable,
        position: usize,
        position_out: usize,
    },
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Add { lhs, rhs, out } => {
                f.write_fmt(format_args!("let {out} = {lhs} + {rhs};"))
            }
            Operator::Sub { lhs, rhs, out } => {
                f.write_fmt(format_args!("let {out} = {lhs} - {rhs};"))
            }
            Operator::Mul { lhs, rhs, out } => {
                f.write_fmt(format_args!("let {out} = {lhs} * {rhs};"))
            }
            Operator::Div { lhs, rhs, out } => {
                f.write_fmt(format_args!("let {out} = {lhs} / {rhs};"))
            }
            Operator::Abs { input, out } => f.write_fmt(format_args!("let {out} = abs({input});")),
            Operator::Exp { input, out } => f.write_fmt(format_args!("let {out} = exp({input});")),
            Operator::Log { input, out } => f.write_fmt(format_args!("let {out} = log({input});")),
            Operator::Powf { lhs, rhs, out } => {
                f.write_fmt(format_args!("let {out} = powf({lhs}, {rhs});"))
            }
            Operator::Log1p { input, out } => {
                f.write_fmt(format_args!("let {out} = log({input} + 1.0);"))
            }
            Operator::Cos { input, out } => f.write_fmt(format_args!("let {out} = cos({input});")),
            Operator::Sin { input, out } => f.write_fmt(format_args!("let {out} = sin({input});")),
            Operator::Tanh { input, out } => {
                f.write_fmt(format_args!("let {out} = tanh({input});"))
            }
            Operator::Erf { input, out } => f.write_fmt(format_args!("let {out} = erf({input});")),
            Operator::AssignGlobal { input, out } => {
                f.write_fmt(format_args!("{out}_global[id] = {input};"))
            }
            Operator::ReadGlobal {
                variable,
                position,
                position_out,
            } => {
                let (global, local) = match variable {
                    Variable::Input(number) => {
                        (format!("input_{number}_global"), format!("input_{number}"))
                    }
                    Variable::Local(_) => panic!("can't read globala local variable."),
                    Variable::Output(number) => (
                        format!("output_{number}_global"),
                        format!("output_{number}"),
                    ),
                    Variable::Scalar(_, _) => panic!("Can't read global scalar variable."),
                };

                f.write_fmt(format_args!(
                    "
var index_{local}: u32 = 0u;

for (var i: u32 = 1u; i <= rank; i++) {{
    let position = {position}u * (2u * rank);
    let position_out = {position_out}u * (2u * rank);

    let stride = info[position + i];
    let stride_out = info[position_out + i];
    let shape = info[position + rank + i];

    index_{local} += id / stride_out % shape * stride;
}}

let {local} = {global}[index_{local}];
"
                ))
            }
        }
    }
}
