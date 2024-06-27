use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Eq)]
pub struct Node {
    value: char,
    frequency: usize,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.frequency.eq(&other.frequency)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.frequency.cmp(&other.frequency).reverse()
    }
}

pub fn build_huff_tree(frequencies: Vec<(&char, &usize)>) -> Option<Box<Node>> {
    let mut heap: BinaryHeap<Box<Node>> = frequencies
        .iter()
        .map(|&(&value, &frequency)| {
            Box::new(Node {
                value,
                frequency,
                left: None,
                right: None,
            })
        })
        .collect();

    while heap.len() > 1 {
        let left = heap.pop()?;
        let right = heap.pop()?;
        let node = Node {
            value: '\0',
            frequency: left.frequency + right.frequency,
            left: Some(left),
            right: Some(right),
        };
        heap.push(Box::new(node));
    }

    heap.pop()
}

pub fn get_huffe(root: &Node) -> HashMap<char, String> {
    fn traverse(node: &Node, code: String, map: &mut HashMap<char, String>) {
        if let Some(left) = &node.left {
            traverse(left, code.clone() + "0", map);
        }
        if let Some(right) = &node.right {
            traverse(right, code.clone() + "1", map);
        }
        if node.value != '\0' {
            map.insert(node.value, code);
        }
    }

    let mut encoded: HashMap<char, String> = HashMap::new();
    traverse(root, String::new(), &mut encoded);

    encoded
}
