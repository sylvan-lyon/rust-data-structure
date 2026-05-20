use aliases::OptionBox;

pub mod traverse;

/// dummy node, it points to root node of the tree
#[derive(Debug, Clone)]
pub struct Tree<T>(pub OptionBox<TreeNode<T>>);

#[derive(Debug, Clone)]
pub struct TreeNode<T> {
    pub value: T,
    pub left: OptionBox<TreeNode<T>>,
    pub right: OptionBox<TreeNode<T>>,
}

pub trait Traverse {
    fn traverse<T>(tree: Option<&TreeNode<T>>, f: &mut impl FnMut(&T));
    fn traverse_mut<T>(tree: Option<&mut TreeNode<T>>, f: &mut impl FnMut(&mut T));
}

impl<T> std::ops::DerefMut for Tree<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> std::ops::Deref for Tree<T> {
    type Target = OptionBox<TreeNode<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Tree<T> {
    pub fn from_slice(s: &[Option<T>]) -> Tree<T>
    where
        T: Clone,
    {
        Tree(TreeNode::from_slice(s))
    }

    pub fn unwrap(self) -> OptionBox<TreeNode<T>> {
        self.0
    }

    pub fn traverse<M: Traverse>(&self, mut f: impl FnMut(&T)) {
        M::traverse(self.0.as_ref().map(Box::as_ref), &mut f)
    }

    pub fn traverse_mut<M: Traverse>(&mut self, mut f: impl FnMut(&mut T)) {
        M::traverse_mut(self.0.as_mut().map(Box::as_mut), &mut f)
    }
}

// constructors
impl<T> TreeNode<T> {
    #[inline]
    pub fn new_boxed(value: T) -> Box<TreeNode<T>> {
        Box::new(Self {
            value: value,
            left: None,
            right: None,
        })
    }

    #[inline]
    pub fn from_slice(s: &[Option<T>]) -> OptionBox<TreeNode<T>>
    where
        T: Clone,
    {
        Self::from_slice_impl(s, 0)
    }

    fn from_slice_impl(s: &[Option<T>], idx: usize) -> OptionBox<TreeNode<T>>
    where
        T: Clone,
    {
        match s.get(idx) {
            Some(Some(x)) => {
                // if slice has element and the first element is not None
                let mut root = Self::new_boxed(x.clone());
                root.left = Self::from_slice_impl(s, 2 * idx + 1);
                root.right = Self::from_slice_impl(s, 2 * idx + 2);
                Some(root)
            }
            _ => None,
        }
    }
}
