// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::max;
use std::ops::Range;

use take_mut::take;

#[derive(Debug)]
pub(crate) struct RangeTree<T: Clone + PartialEq> {
    root: Option<Box<Node<T>>>,
}

impl<T: Clone + PartialEq> RangeTree<T> {
    pub(crate) fn new() -> RangeTree<T> {
        RangeTree { root: None }
    }

    pub(crate) fn insert(&mut self, old_range: Range<usize>, new_len: usize, data: T) {
        assert!(new_len > 0);
        if let Some(root) = self.root.as_mut() {
            root.insert(old_range, new_len, data)
        } else {
            assert!(old_range.start == 0 && old_range.is_empty());
            self.root = Some(Box::new(Node::new_leaf(new_len, data)))
        }
    }

    pub(crate) fn remove(&mut self, range: Range<usize>) {
        if let Some(root) = &mut self.root {
            assert!(range.end <= root.len);
            if range.len() == root.len {
                self.root = None;
            } else {
                root.remove(range);
            }
        } else {
            panic!("deleting from empty tree")
        }
    }

    pub(crate) fn iter(&self) -> Option<RangeTreeIter<T>> {
        self.iter_range(0..self.len())
    }

    pub(crate) fn iter_range(&self, range: Range<usize>) -> Option<RangeTreeIter<T>> {
        self.root.as_ref().map(|x| RangeTreeIter::new(x, range))
    }

    pub(crate) fn len(&self) -> usize {
        self.root.as_ref().map(|x| x.len).unwrap_or(0)
    }
}

#[derive(Debug)]
struct LeafNode<T: Clone + PartialEq> {
    data: T,
}

#[derive(Debug)]
struct InnerNode<T: Clone + PartialEq> {
    left: Box<Node<T>>,
    right: Box<Node<T>>,
    height: usize,
}

#[derive(Debug)]
enum NodeTyp<T: Clone + PartialEq> {
    Leaf(LeafNode<T>),
    Inner(InnerNode<T>),
}

#[derive(Debug)]
struct Node<T: Clone + PartialEq> {
    len: usize,
    typ: NodeTyp<T>,
}

impl<T: Clone + PartialEq> Node<T> {
    fn new_leaf(len: usize, data: T) -> Node<T> {
        assert!(len > 0);
        Node {
            len,
            typ: NodeTyp::Leaf(LeafNode { data }),
        }
    }

    fn new_inner(left: Box<Node<T>>, right: Box<Node<T>>) -> Node<T> {
        let len = left.len + right.len;
        assert!(len > 0);
        // Rebalance if required
        let (left, right) = if left.height() > right.height() + 1 {
            Node::right_rotate(left, right)
        } else if right.height() > left.height() + 1 {
            Node::left_rotate(left, right)
        } else {
            (left, right)
        };
        let height = 1 + max(left.height(), right.height());
        Node {
            len,
            typ: NodeTyp::Inner(InnerNode {
                left,
                right,
                height,
            }),
        }
    }

    fn insert(&mut self, old_range: Range<usize>, new_len: usize, new_data: T) {
        assert!(old_range.end <= self.len);
        let range_len = old_range.len();
        take(self, |node| match node.typ {
            NodeTyp::Leaf(leaf) => {
                if leaf.data == new_data {
                    Node::new_leaf(node.len - range_len + new_len, leaf.data)
                } else {
                    let left = if old_range.start > 0 {
                        Node::new_inner(
                            Box::new(Node::new_leaf(old_range.start, leaf.data.clone())),
                            Box::new(Node::new_leaf(new_len, new_data)),
                        )
                    } else {
                        Node::new_leaf(new_len, new_data)
                    };
                    if old_range.end < node.len {
                        Node::new_inner(
                            Box::new(left),
                            Box::new(Node::new_leaf(node.len - old_range.end, leaf.data)),
                        )
                    } else {
                        left
                    }
                }
            }
            NodeTyp::Inner(mut inner) => {
                let left_len = inner.left.len;
                if old_range.end <= left_len {
                    inner.left.insert(old_range, new_len, new_data);
                    Node::new_inner(inner.left, inner.right)
                } else if old_range.start >= left_len {
                    let range = old_range.start - left_len..old_range.end - left_len;
                    inner.right.insert(range, new_len, new_data);
                    Node::new_inner(inner.left, inner.right)
                } else {
                    inner
                        .left
                        .insert(old_range.start..left_len, new_len, new_data);
                    if old_range.end < node.len {
                        inner.right.remove(0..old_range.end - left_len);
                        Node::new_inner(inner.left, inner.right)
                    } else {
                        *inner.left
                    }
                }
            }
        });
    }

    fn remove(&mut self, range: Range<usize>) {
        assert!(!range.is_empty());
        assert!(range.len() < self.len);
        take(self, |node| match node.typ {
            NodeTyp::Leaf(leaf) => Node::new_leaf(node.len - range.len(), leaf.data),
            NodeTyp::Inner(mut inner) => {
                let left_len = inner.left.len;
                if range.end <= left_len {
                    if range.start == 0 && range.end == left_len {
                        *inner.right
                    } else {
                        inner.left.remove(range);
                        Node::new_inner(inner.left, inner.right)
                    }
                } else if range.start >= left_len {
                    if range.start == left_len && range.end == node.len {
                        *inner.left
                    } else {
                        let range = range.start - left_len..range.end - left_len;
                        inner.right.remove(range);
                        Node::new_inner(inner.left, inner.right)
                    }
                } else if range.start == 0 {
                    inner.right.remove(0..range.end - left_len);
                    *inner.right
                } else if range.end == node.len {
                    inner.left.remove(range.start..left_len);
                    *inner.left
                } else {
                    inner.right.remove(0..range.end - left_len);
                    inner.left.remove(range.start..left_len);
                    Node::new_inner(inner.left, inner.right)
                }
            }
        })
    }

    fn height(&self) -> usize {
        match &self.typ {
            NodeTyp::Leaf(_) => 1,
            NodeTyp::Inner(inner) => inner.height,
        }
    }

    fn right_rotate(left: Box<Node<T>>, right: Box<Node<T>>) -> (Box<Node<T>>, Box<Node<T>>) {
        let (ll, lr) = match left.typ {
            NodeTyp::Inner(inner) => {
                if inner.right.height() > inner.left.height() {
                    let (lrl, lrr) = match inner.right.typ {
                        NodeTyp::Inner(inner) => (inner.left, inner.right),
                        NodeTyp::Leaf(_) => unreachable!(),
                    };
                    (Box::new(Node::new_inner(inner.left, lrl)), lrr)
                } else {
                    (inner.left, inner.right)
                }
            }
            NodeTyp::Leaf(_) => unreachable!(),
        };
        (ll, Box::new(Node::new_inner(lr, right)))
    }

    fn left_rotate(left: Box<Node<T>>, right: Box<Node<T>>) -> (Box<Node<T>>, Box<Node<T>>) {
        let (rl, rr) = match right.typ {
            NodeTyp::Inner(inner) => {
                if inner.left.height() > inner.right.height() {
                    let (rll, rlr) = match inner.left.typ {
                        NodeTyp::Inner(inner) => (inner.left, inner.right),
                        NodeTyp::Leaf(_) => unreachable!(),
                    };
                    (rll, Box::new(Node::new_inner(rlr, inner.right)))
                } else {
                    (inner.left, inner.right)
                }
            }
            NodeTyp::Leaf(_) => unreachable!(),
        };
        (Box::new(Node::new_inner(left, rl)), rr)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RangeTreeIter<'a, T: Clone + PartialEq> {
    stack: Vec<&'a InnerNode<T>>,
    cur_start: usize,
    cur_end: usize,
    last_end: usize,
    cur_node: Option<&'a LeafNode<T>>,
}

impl<'a, T: Clone + PartialEq> RangeTreeIter<'a, T> {
    fn new(root: &'a Node<T>, mut range: Range<usize>) -> RangeTreeIter<'a, T> {
        if range.is_empty() {
            return RangeTreeIter {
                stack: vec![],
                cur_start: 0,
                cur_end: 0,
                last_end: 0,
                cur_node: None,
            };
        }
        assert!(range.end <= root.len);
        let mut node = root;
        let mut stack = Vec::new();
        let cur_start = range.start;
        let cur_end;
        let cur_node;
        loop {
            match &node.typ {
                NodeTyp::Leaf(leaf) => {
                    cur_node = Some(leaf);
                    cur_end = cur_start + node.len - range.start;
                    break;
                }
                NodeTyp::Inner(inner) => {
                    if inner.left.len > range.start {
                        stack.push(inner);
                        node = &inner.left;
                    } else {
                        range.start -= inner.left.len;
                        node = &inner.right;
                    }
                }
            }
        }
        RangeTreeIter {
            stack,
            cur_start,
            last_end: range.end,
            cur_end,
            cur_node,
        }
    }
}

impl<'a, T: Clone + PartialEq> Iterator for RangeTreeIter<'a, T> {
    type Item = (Range<usize>, &'a T);

    fn next(&mut self) -> Option<(Range<usize>, &'a T)> {
        if let Some(ret_node) = self.cur_node.take() {
            let mut range = self.cur_start..self.cur_end;
            self.cur_start = self.cur_end;
            if let Some(last) = self.stack.pop() {
                let mut node = &last.right;
                loop {
                    match &node.typ {
                        NodeTyp::Leaf(leaf) => {
                            self.cur_node = Some(leaf);
                            self.cur_end = self.cur_start + node.len;
                            break;
                        }
                        NodeTyp::Inner(inner) => {
                            self.stack.push(inner);
                            node = &inner.left;
                        }
                    }
                }
            }
            if range.end >= self.last_end {
                range.end = self.last_end;
                self.cur_node = None;
            }
            Some((range, &ret_node.data))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nums(tree: &RangeTree<i32>) -> Vec<(Range<usize>, i32)> {
        tree.iter()
            .unwrap()
            .map(|(x, y)| (x, y.clone()))
            .collect::<Vec<_>>()
    }

    #[test]
    fn iter_range() {
        let mut tree = RangeTree::new();
        tree.insert(0..0, 50, 1);
        tree.insert(5..10, 2, 2);
        tree.insert(12..17, 2, 3);
        tree.insert(19..24, 2, 4);
        tree.insert(13..20, 20, 5);
        tree.insert(49..54, 20, 6);
        tree.insert(69..69, 20, 7);
        assert_eq!(
            tree.iter_range(14..71)
                .unwrap()
                .map(|(x, y)| (x, y.clone()))
                .collect::<Vec<_>>(),
            vec![
                (14..33, 5),
                (33..34, 4),
                (34..49, 1),
                (49..69, 6),
                (69..71, 7),
            ]
        );
    }

    #[test]
    fn insert_only() {
        let mut tree = RangeTree::new();

        tree.insert(0..0, 50, 1);
        assert_eq!(nums(&tree), vec![(0..50, 1)]);

        tree.insert(5..10, 2, 2);
        assert_eq!(nums(&tree), vec![(0..5, 1), (5..7, 2), (7..47, 1)]);

        tree.insert(12..17, 2, 3);
        assert_eq!(
            nums(&tree),
            vec![(0..5, 1), (5..7, 2), (7..12, 1), (12..14, 3), (14..44, 1)]
        );

        tree.insert(19..24, 2, 4);
        assert_eq!(
            nums(&tree),
            vec![
                (0..5, 1),
                (5..7, 2),
                (7..12, 1),
                (12..14, 3),
                (14..19, 1),
                (19..21, 4),
                (21..41, 1)
            ]
        );

        tree.insert(13..20, 20, 5);
        assert_eq!(
            nums(&tree),
            vec![
                (0..5, 1),
                (5..7, 2),
                (7..12, 1),
                (12..13, 3),
                (13..33, 5),
                (33..34, 4),
                (34..54, 1)
            ]
        );

        tree.insert(49..54, 20, 6);
        assert_eq!(
            nums(&tree),
            vec![
                (0..5, 1),
                (5..7, 2),
                (7..12, 1),
                (12..13, 3),
                (13..33, 5),
                (33..34, 4),
                (34..49, 1),
                (49..69, 6),
            ]
        );

        tree.insert(69..69, 20, 7);
        assert_eq!(
            nums(&tree),
            vec![
                (0..5, 1),
                (5..7, 2),
                (7..12, 1),
                (12..13, 3),
                (13..33, 5),
                (33..34, 4),
                (34..49, 1),
                (49..69, 6),
                (69..89, 7),
            ]
        );
    }

    #[test]
    fn insert_remove() {
        let mut tree = RangeTree::new();
        tree.insert(0..0, 50, 1);
        tree.insert(5..10, 2, 2);
        tree.insert(12..17, 2, 3);
        tree.insert(19..24, 2, 4);
        tree.insert(13..20, 20, 5);
        tree.insert(49..54, 20, 6);
        tree.insert(69..69, 20, 7);

        tree.remove(1..3);
        assert_eq!(
            nums(&tree),
            vec![
                (0..3, 1),
                (3..5, 2),
                (5..10, 1),
                (10..11, 3),
                (11..31, 5),
                (31..32, 4),
                (32..47, 1),
                (47..67, 6),
                (67..87, 7),
            ]
        );

        tree.remove(2..4);
        assert_eq!(
            nums(&tree),
            vec![
                (0..2, 1),
                (2..3, 2),
                (3..8, 1),
                (8..9, 3),
                (9..29, 5),
                (29..30, 4),
                (30..45, 1),
                (45..65, 6),
                (65..85, 7),
            ]
        );

        tree.remove(10..35);
        assert_eq!(
            nums(&tree),
            vec![
                (0..2, 1),
                (2..3, 2),
                (3..8, 1),
                (8..9, 3),
                (9..10, 5),
                (10..20, 1),
                (20..40, 6),
                (40..60, 7),
            ]
        );

        tree.remove(0..60);
        assert!(tree.iter().is_none());
    }
}
