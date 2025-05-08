use crate::ir::{ArgType, AttributeValue, Data, Node, TensorType};

/// Create a ReduceSumConfig from the attributes of the node
pub fn reduce_sum_config(node: &Node) -> Option<usize> {
    let mut axes = Vec::new();
    let mut keepdims = 1;

    let tensor = match node.inputs.first().unwrap().clone().ty {
        ArgType::Tensor(tensor) => tensor,
        _ => panic!("Only tensor input is valid"),
    };

    // Extract the attributes
    for (key, value) in node.attrs.iter() {
        match key.as_str() {
            "keepdims" => keepdims = value.clone().into_i64(),
            "axes" => axes = value.clone().into_i64s(),
            // TODO: handle noop_with_empty_axes
            _ => {}
        }
    }

    // Process axes from additional input (if available)
    if let Some(value) = node
        .inputs
        .get(1)
        .and_then(|argument| argument.value.as_ref())
    {
        axes = value.clone().data.into_i64s();
    }

    if axes.len() > 1 {
        panic!("ReduceSum: reducing on multiple dimensions is not supported")
    }

    if axes.is_empty() && keepdims == 1 {
        panic!("ReduceSum: axes must be provided with keepdims")
    }

    if !axes.is_empty() && keepdims == 0 {
        // Not supported in Burn
        panic!("ReduceSum: the reduce operation must preserve the reduced dimension")
    }

    if axes.is_empty() {
        None
    } else {
        let mut dim = axes[0];

        if dim < 0 {
            // Accepted range is [-r, r-1] where r = rank(data) but Burn only supports positive dim
            dim += tensor.rank as i64;
        }
        Some(dim as usize)
    }
}

/// Update output rank for ReduceSum based on axes.
pub fn reduce_sum_update_outputs(node: &mut Node) {
    log::debug!("ReduceSum rank inference for node {}", node.name);

    let tensor = match &node.inputs[0].ty {
        ArgType::Tensor(tensor) => tensor,
        _ => panic!("Only tensor input is valid"),
    };
    log::debug!("ReduceSum input rank for {}: {}", node.name, tensor.rank);

    let dim_only = match node.attrs.get("axes") {
        Some(value) => match &value {
            AttributeValue::Int64(_) => true,
            AttributeValue::Int64s(ints) => ints.len() == 1,
            _ => false,
        },
        None => false,
    } || match node.inputs.get(1).and_then(|arg| arg.value.as_ref()) {
        Some(value) => match &value.data {
            Data::Int64(_) => true,
            Data::Int64s(ints) => ints.len() == 1,
            _ => false,
        },
        None => false,
    };

    let output_rank = if dim_only { tensor.rank } else { 1 };
    log::debug!("ReduceSum output rank for {}: {}", node.name, output_rank);

    node.outputs[0].ty = ArgType::Tensor(TensorType {
        elem_type: tensor.elem_type.clone(),
        rank: output_rank,
        static_shape: None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::NodeType;
    use crate::node::test_utils::NodeBuilder;

    fn create_test_node(
        axes: Option<Vec<i64>>,
        keepdims: Option<i64>,
        with_axes_input: bool,
    ) -> Node {
        let mut builder = NodeBuilder::new(NodeType::ReduceSum, "test_reduce_sum")
            .input_tensor_f32("data", 3, None)
            .output_tensor_f32("reduced", 3, None);

        // Add axes input if requested
        if with_axes_input && axes.is_some() {
            let axes_vec = axes.clone().unwrap();
            builder = builder.input_tensor_i64_data("axes", axes_vec.clone(), vec![axes_vec.len()]);
        }

        // Add attributes
        if !with_axes_input && axes.is_some() {
            builder = builder.attr_ints("axes", axes.clone().unwrap());
        }

        if let Some(kd) = keepdims {
            builder = builder.attr_int("keepdims", kd);
        }

        builder.build()
    }

    #[test]
    fn test_reduce_sum_config_basic() {
        let node = create_test_node(Some(vec![1]), Some(1), false);
        let dim = reduce_sum_config(&node);
        assert_eq!(dim, Some(1));
    }

    #[test]
    fn test_reduce_sum_config_with_input_axes() {
        let node = create_test_node(Some(vec![1]), Some(1), true);
        let dim = reduce_sum_config(&node);
        assert_eq!(dim, Some(1));
    }

    #[test]
    fn test_reduce_sum_config_negative_axis() {
        let node = create_test_node(Some(vec![-2]), Some(1), false);
        let dim = reduce_sum_config(&node);
        assert_eq!(dim, Some(1)); // -2 + 3 = 1
    }

    #[test]
    #[should_panic(expected = "ReduceSum: axes must be provided with keepdims")]
    fn test_reduce_sum_config_no_axes() {
        let node = create_test_node(None, Some(1), false);
        let _ = reduce_sum_config(&node);
    }

    #[test]
    #[should_panic(expected = "ReduceSum: reducing on multiple dimensions is not supported")]
    fn test_reduce_sum_config_multiple_axes() {
        let node = create_test_node(Some(vec![0, 1]), Some(1), false);
        let _ = reduce_sum_config(&node);
    }

    #[test]
    #[should_panic(
        expected = "ReduceSum: the reduce operation must preserve the reduced dimension"
    )]
    fn test_reduce_sum_config_no_keepdims() {
        let node = create_test_node(Some(vec![1]), Some(0), false);
        let _ = reduce_sum_config(&node);
    }
}
