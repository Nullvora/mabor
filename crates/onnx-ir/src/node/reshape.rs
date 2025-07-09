use crate::ir::{ArgType, Data, Node, TensorData, TensorType};

/// Update output rank for Reshape based on shape input if constant, otherwise use input rank.
pub fn reshape_update_outputs(node: &mut Node) {
    log::debug!("Reshape rank inference for node {}", node.name);

    let shape = if node.inputs.len() == 2 {
        log::debug!("Reshape node {} has shape as second input", node.name);
        match &node.inputs[1].value {
            Some(value) => match &value.data {
                Data::Int64s(shape) => {
                    log::debug!("Reshape node {} has constant shape: {:?}", node.name, shape);
                    Some(shape.clone())
                }
                _ => panic!("Reshape: invalid input types"),
            },
            None => {
                log::debug!(
                    "Reshape node {} has dynamic shape as second input",
                    node.name
                );
                None
            }
        }
    } else {
        log::debug!("Reshape node {} using shape from attributes", node.name);
        node.attrs.get("shape").cloned().map(|v| {
            let shape = v.into_i64s();
            log::debug!("Reshape node {} shape attribute: {:?}", node.name, shape);
            shape
        })
    };

    let output = match &node.outputs[0].ty {
        ArgType::Tensor(tensor) => tensor.clone(),
        _ => panic!("Reshape: invalid output types"),
    };

    let rank = match &shape {
        Some(s) => s.len(),
        None => output.rank,
    };

    log::debug!("Reshape output rank for node {}: {}", node.name, rank);

    node.outputs[0].ty = ArgType::Tensor(TensorType {
        rank,
        static_shape: None,
        ..output
    });
}

pub fn reshape_config(node: &Node) -> Vec<i64> {
    let mut allowzero = 0;

    for (key, value) in node.attrs.iter() {
        match key.as_str() {
            "allowzero" => allowzero = value.clone().into_i64(),
            "shape" => {} // This can be used when shape is not provided as input - handled elsewhere
            _ => panic!("Unexpected attribute for Reshape: {key}"),
        }
    }

    // Burn does not support zero size shape (0 means false in ONNX)
    // (see https://onnx.ai/onnx/operators/onnx__Reshape.html#attributes)
    if allowzero != 0 {
        panic!("Zero shape size is not supported");
    }

    // TODO: check "shape" attribute
    if node.inputs.len() != 2 || node.inputs[1].value.is_none() {
        panic!("Reshape: shape tensor must be present for {node:?}");
    }

    match &node.inputs[1].value {
        Some(TensorData { data, shape, .. }) => {
            assert_eq!(shape.len(), 1, "Reshape: shape tensor must be 1D");
            data.clone().into_i64s()
        }
        _ => panic!("Only tensor input is valid for shape"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::NodeType;
    use crate::node::test_utils::NodeBuilder;

    fn create_test_node(allowzero: i64, shape_vec: Vec<i64>) -> Node {
        let mut builder = NodeBuilder::new(NodeType::Reshape, "test_reshape")
            .input_tensor_f32("data", 4, None)
            .input_tensor_i64_data("shape", shape_vec.clone(), vec![shape_vec.len()])
            .output_tensor_f32("reshaped", 2, None);

        if allowzero != 0 {
            builder = builder.attr_int("allowzero", allowzero);
        }

        builder.build()
    }

    #[test]
    fn test_reshape_config_basic() {
        let node = create_test_node(0, vec![2, 3]);
        let shape = reshape_config(&node);
        assert_eq!(shape, vec![2, 3]);
    }

    #[test]
    #[should_panic(expected = "Zero shape size is not supported")]
    fn test_reshape_config_allowzero_not_supported() {
        let node = create_test_node(1, vec![2, 3]);
        let _ = reshape_config(&node);
    }

    #[test]
    #[should_panic(expected = "shape tensor must be present")]
    fn test_reshape_config_no_shape_input() {
        let mut node = create_test_node(0, vec![2, 3]);
        node.inputs.pop(); // Remove the shape input
        let _ = reshape_config(&node);
    }

    #[test]
    #[should_panic(expected = "shape tensor must be 1D")]
    fn test_reshape_config_invalid_shape_dim() {
        let mut node = create_test_node(0, vec![2, 3]);
        // Modify the shape tensor's shape to be 2D
        if let Some(tensor_data) = &mut node.inputs[1].value {
            tensor_data.shape = vec![2, 1];
        }
        let _ = reshape_config(&node);
    }
}
