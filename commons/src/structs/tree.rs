use std::{cell::RefCell, rc::Rc};

pub type OptionalNode = Option<Rc<RefCell<TreeNode>>>;

/**
 * TreeNode
 */
#[derive(Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub val: i32,
    pub left: OptionalNode,
    pub right: OptionalNode,
}

impl TreeNode {
    // Create new TreeNode
    #[inline]
    pub fn new(val: i32) -> Self {
        Self {
            val,
            left: None,
            right: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TreeNode;
    use std::{cell::RefCell, rc::Rc};
    #[test]
    fn test_tree() {
        let mut root = TreeNode::new(1);
        root.left = Option::Some(Rc::new(RefCell::new(TreeNode::new(2))));
        root.right = None;

        // use RefCell to attain mutability
        if let Some(l) = root.left {
            l.borrow_mut().left = None;
        }
    }
}
