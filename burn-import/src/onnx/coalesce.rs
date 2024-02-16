use std::{
    cell::{RefCell, RefMut},
    collections::{HashMap, HashSet},
    iter::Peekable,
    slice::{Iter, IterMut},
};

use super::{
    ir::{AttributeValue, Node, NodeType},
    proto_conversion::convert_node_proto,
    protos::{NodeProto, TensorProto},
};
use crate::onnx::{
    from_onnx::move_initializer_data,
    ir::{ArgType, Data, TensorType},
};

/// The function transforms the graph into a new one where the nodes are coalesced into a single node.
pub fn coalesce(
    node: &mut Node,
    nodes_iter: &mut Peekable<Iter<NodeProto>>,
    initializers: &HashMap<String, TensorProto>,
) {
    match node.node_type {
        NodeType::Gemm => convert_gemm_to_linear(node),
        NodeType::MatMul => {
            convert_matmul_to_linear(node, nodes_iter, initializers);
        }
        _ => {}
    }
}

/// This function converts a Gemm node into a Linear node
///
/// PyTorch and other frameworks use Gemm node to represent Linear layer.
pub(crate) fn convert_gemm_to_linear(node: &mut Node) {
    if node.outputs.len() != 1 {
        panic!("Gemm node must have 1 output");
    }
    let straight_linear = match (
        node.attrs.get("alpha"),
        node.attrs.get("beta"),
        node.attrs.get("transB"),
    ) {
        (
            Some(AttributeValue::Float32(alpha)),
            Some(AttributeValue::Float32(beta)),
            Some(AttributeValue::Int64(trans_b)),
        ) => *alpha == 1.0 && *beta == 1.0 && *trans_b == 1,
        _ => false,
    };

    if straight_linear {
        node.node_type = NodeType::Linear;
        node.attrs.remove("alpha");
        node.attrs.remove("beta");
        node.attrs.remove("transB");

        // Transpose the weights
        transpose_linear_node_weights(node);
    } else {
        panic!("Full Gemm node not supported yet.");
    }
}

// Transpose linear weights (required for Gemm -> Linear conversion)
fn transpose_linear_node_weights(node: &mut Node) {
    assert!(
        node.inputs.len() > 1,
        "Linear node must have at least 2 input"
    );

    assert!(node.inputs[1].value.is_some(), "Input must have a value");

    let weight = node.inputs[1]
        .clone()
        .into_tensor()
        .expect("Tensor input is expected");

    assert_eq!(weight.dim, 2, "Weight must be a 2D tensor");

    let shape = weight.shape.unwrap();

    match weight.data.expect("Tensor must have data") {
        Data::Float32s(data) => {
            let data_t = transpose_flattened(data, shape[0], shape[1]);
            node.inputs[1].value = Some(Data::Float32s(data_t));
        }
        Data::Float64s(data) => {
            let data_t = transpose_flattened(data, shape[0], shape[1]);
            node.inputs[1].value = Some(Data::Float64s(data_t));
        }
        Data::Float16s(data) => {
            let data_t = transpose_flattened(data, shape[0], shape[1]);
            node.inputs[1].value = Some(Data::Float16s(data_t));
        }
        _ => panic!("Only float types are supported for Linear node"),
    }
    let shape = Some(vec![shape[1], shape[0]]); // Transpose the shape
    node.inputs[1].ty = ArgType::Tensor(TensorType {
        shape,
        elem_type: weight.elem_type,
        dim: 2,
    });
}

fn transpose_flattened<T: Copy>(matrix: Vec<T>, rows: usize, cols: usize) -> Vec<T> {
    assert_eq!(matrix.len(), rows * cols, "Matrix must be flattened");

    let mut transposed: Vec<T> = vec![matrix[0]; matrix.len()];

    for i in 0..rows {
        for j in 0..cols {
            transposed[j * rows + i] = matrix[i * cols + j];
        }
    }

    transposed
}

/// This function converts a MatMul node into a Linear node if possible.
///
/// PyTorch and other frameworks use MatMul node to represent Linear layer.
///
/// This function also converts the following Add node into a Linear node if possible.
/// Add node is used to represent bias in PyTorch.
pub(crate) fn convert_matmul_to_linear(
    node: &mut Node,
    iter_mut: &mut Peekable<Iter<NodeProto>>,
    initializers: &HashMap<String, TensorProto>,
) {
    if node.inputs.len() != 2 {
        panic!("MatMul node must have 2 inputs");
    }

    // if the second input does not have a value, it is not a weight, then proceed to the next node
    if node.inputs[1].value.is_none() {
        return;
    }

    // Check if the second input is a 2D tensor
    if let ArgType::Tensor(ref tensor_type) = node.inputs[1].ty {
        assert_eq!(tensor_type.dim, 2, "Weight must be a 2D tensor");
    } else {
        panic!("Tensor input is expected");
    }

    // Convert the node to Linear
    node.node_type = NodeType::Linear;

    // Check the next node for potential conversion
    if let Some(peek_node) = iter_mut.peek() {
        let mut peek_node = convert_node_proto(peek_node).clone();
        for node_input in peek_node.inputs.iter_mut() {
            // self.input_of
            //     .entry(node_input.name.clone())
            //     .and_modify(|f| f.push(i))
            //     .or_insert(vec![i]);
            if let Some(initializer) = initializers.get(&node_input.name) {
                move_initializer_data(initializer, node_input);
            }
        }
        println!("next node is {:?}", peek_node);
        if is_add_node_with_bias(&peek_node, node) {
            convert_and_remove_add_node(&peek_node, node);

            // You don't have to remove it if it's never stored in the first place
            let _ = iter_mut.next();
            println!("\n\nskipping add node\n\n");
        }
    }
}

/// This function converts a MatMul node into a Linear node if possible.
///
/// PyTorch and other frameworks use MatMul node to represent Linear layer.
///
/// This function also converts the following Add node into a Linear node if possible.
/// Add node is used to represent bias in PyTorch.
pub(crate) fn convert_matmul_to_linear2(
    node_vec: &Vec<RefCell<Node>>,
    node_index: usize,
    nodes_to_remove: &mut HashSet<usize>,
) {
    let mut node = node_vec[node_index].borrow_mut();
    if node.inputs.len() != 2 {
        panic!("MatMul node must have 2 inputs");
    }

    // if the second input does not have a value, it is not a weight, then proceed to the next node
    if node.inputs[1].value.is_none() {
        return;
    }

    // Check if the second input is a 2D tensor
    if let ArgType::Tensor(ref tensor_type) = node.inputs[1].ty {
        assert_eq!(tensor_type.dim, 2, "Weight must be a 2D tensor");
    } else {
        panic!("Tensor input is expected");
    }

    // Convert the node to Linear
    node.node_type = NodeType::Linear;

    // Check the next node for potential conversion

    if node_index + 1 < node_vec.len() {
        let next_node = node_vec[node_index + 1].borrow();
        if is_add_node_with_bias(&next_node, &node) {
            convert_node2(&next_node, node);
            nodes_to_remove.insert(node_index + 1);
        }
    }
}
/// Helper function to check if the peeked node is an Add node with bias
fn is_add_node_with_bias(peek_node: &Node, current_node: &Node) -> bool {
    if (peek_node.node_type == NodeType::Add && peek_node.inputs.len() == 2) {
        println!("\n\ntwo matches");
        println!("peek_node.inputs[0].name: {:?}", peek_node.inputs[0].name);
        println!(
            "current_node.outputs[0].name: {:?}",
            current_node.outputs[0].name
        );
        return ((peek_node.inputs[0].name == current_node.outputs[0].name
            && peek_node.inputs[1].value.is_some())
            || (peek_node.inputs[1].name == current_node.outputs[0].name
                && peek_node.inputs[0].value.is_some()));
    }
    false
}

/// Helper function to convert and remove the Add node
fn convert_and_remove_add_node(bias_node: &Node, current_node: &mut Node) {
    let bias_input = if bias_node.inputs[0].value.is_some() {
        bias_node.inputs[0].clone()
    } else {
        bias_node.inputs[1].clone()
    };

    // Push the bias input and update the output name
    current_node.inputs.push(bias_input);
    current_node.outputs[0].name = bias_node.outputs[0].name.clone();
}

/// Helper function to convert and remove the Add node
pub(crate) fn convert_node2<'parser>(bias_node: &Node, mut current_node: RefMut<'parser, Node>) {
    let bias_input = if bias_node.inputs[0].value.is_some() {
        bias_node.inputs[0].clone()
    } else {
        bias_node.inputs[1].clone()
    };

    // Push the bias input and update the output name
    current_node.inputs.push(bias_input);
    current_node.outputs[0].name = bias_node.outputs[0].name.clone();
}
