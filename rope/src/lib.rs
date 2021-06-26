use std::io::{Read, Result as IOResult};
use std::ops::RangeBounds;

mod builder;
mod cow_box;

use cow_box::CowBox;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rope {
    root: CowBox<Node>,
}

impl Rope {
    pub fn new() -> Rope {
        Rope {
            root: CowBox::new(Node::new_leaf(LeafNode::new("".to_owned()))),
        }
    }

    pub fn from_reader<R: Read>(reader: R) -> IOResult<Rope> {
        builder::RopeBuilder::from_reader(reader).map(|builder| builder.build())
    }

    pub fn slice<'a, R: RangeBounds<usize>>(&'a self, _range: R) -> RopeSlice<'a> {
        unimplemented!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RopeSlice<'a> {
    rope: &'a Rope,
    start_offset: usize,
    end_offset: usize,
}

impl<'a> RopeSlice<'a> {
    pub fn slice<R: RangeBounds<usize>>(_range: R) -> RopeSlice<'a> {
        unimplemented!()
    }

    pub fn to_string(&self) -> String {
        unimplemented!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Node {
    typ: NodeTyp,
}

impl Node {
    fn new_inner(inner_node: InnerNode) -> Node {
        Node {
            typ: NodeTyp::Inner(inner_node),
        }
    }

    fn new_leaf(leaf_node: LeafNode) -> Node {
        Node {
            typ: NodeTyp::Leaf(leaf_node),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum NodeTyp {
    Inner(InnerNode),
    Leaf(LeafNode),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InnerNode {
    left: CowBox<Node>,
    right: CowBox<Node>,
}

impl InnerNode {
    fn new(left: Node, right: Node) -> InnerNode {
        InnerNode {
            left: CowBox::new(left),
            right: CowBox::new(right),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LeafNode {
    data: String,
}

impl LeafNode {
    fn new(data: String) -> LeafNode {
        LeafNode { data }
    }
}

const MAX_NODE_SIZE: usize = 4096;
