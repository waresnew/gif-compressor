use std::{
    cell::{Ref, RefCell},
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
    rc::Rc,
};

type MaybeNode<T> = Option<Box<Node<T>>>;

pub trait Point<const K: usize> {
    ///from 0 to K-1
    fn get(&self, dim: usize) -> i32;
    fn distance(&self, other: &Self) -> usize {
        let mut ans = 0;
        for i in 0..K {
            let next = self.get(i) - other.get(i);
            ans += next * next;
        }
        ans as usize
    }
}
#[derive(Debug)]
struct Node<T: Copy> {
    val: T,
    left: MaybeNode<T>,
    right: MaybeNode<T>,
}

/// pair but only the first value is used in `Eq` and `Ord`
pub struct PairFirstOnly<X: Ord + Eq, Y> {
    pub first: X,
    pub second: Y,
}
impl<X: Ord + Eq, Y> PairFirstOnly<X, Y> {
    pub fn new(x: X, y: Y) -> Self {
        Self {
            first: x,
            second: y,
        }
    }
}
impl<X: Ord + Eq, Y> Ord for PairFirstOnly<X, Y> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.first.cmp(&other.first)
    }
}
impl<X: Ord + Eq, Y> PartialOrd for PairFirstOnly<X, Y> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<X: Ord + Eq, Y> PartialEq for PairFirstOnly<X, Y> {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first
    }
}
impl<X: Ord + Eq, Y> Eq for PairFirstOnly<X, Y> {}
#[derive(Debug)]
///dynamic insert/delete not implemented
pub struct KdTree<T: Copy, const K: usize> {
    root: MaybeNode<T>,
    size: usize,
    cache: RefCell<HashMap<T, Rc<Vec<T>>>>,
}

impl<T, const K: usize> KdTree<T, K>
where
    T: Point<K> + Copy + Eq + Hash,
{
    pub fn new(mut lst: Vec<T>) -> Self {
        Self {
            size: lst.len(),
            root: Self::make_subtree(&mut lst, 0),
            cache: RefCell::new(HashMap::new()),
        }
    }
    fn make_subtree(lst: &mut [T], depth: usize) -> MaybeNode<T> {
        if lst.is_empty() {
            return None;
        }
        let dim = Self::get_dim(depth);
        let mid = lst.len() / 2;
        let split = lst.select_nth_unstable_by(mid, |a, b| a.get(dim).cmp(&b.get(dim)));
        Some(Box::new(Node {
            val: *split.1,
            left: Self::make_subtree(split.0, depth + 1),
            right: Self::make_subtree(split.2, depth + 1),
        }))
    }
    fn get_dim(depth: usize) -> usize {
        depth % K
    }
    fn k_nn_helper(
        cur: &MaybeNode<T>,
        target: T,
        k: usize,
        depth: usize,
        ans: &mut BinaryHeap<PairFirstOnly<usize, T>>,
    ) {
        let Some(cur) = cur else {
            return;
        };
        let dim = Self::get_dim(depth);
        let mut went_left = false;
        let dis = target.distance(&cur.val);
        ans.push(PairFirstOnly::new(dis, cur.val));
        if ans.len() > k {
            ans.pop();
        }
        match target.get(dim).cmp(&cur.val.get(dim)) {
            Ordering::Less | Ordering::Equal => {
                went_left = true;
                Self::k_nn_helper(&cur.left, target, k, depth + 1, ans)
            }
            Ordering::Greater => Self::k_nn_helper(&cur.right, target, k, depth + 1, ans),
        }
        let max_dis = ans.peek().unwrap().first;
        if ans.len() < k
            || max_dis > (target.get(dim) - cur.val.get(dim)).unsigned_abs().pow(2) as usize
        {
            Self::k_nn_helper(
                if went_left { &cur.right } else { &cur.left },
                target,
                k,
                depth + 1,
                ans,
            );
        }
    }
    ///panics if k>kdtree size
    ///this implies that if k==0, then the tree must be empty too
    pub fn k_nn(&self, target: T, k: usize) -> Rc<Vec<T>> {
        if k > self.size {
            panic!("k>kdtree size: k={}, tree size={}", k, self.size);
        }
        self.cache.borrow_mut().entry(target).or_insert_with(|| {
            let mut heap: BinaryHeap<PairFirstOnly<usize, T>> = BinaryHeap::with_capacity(k);
            Self::k_nn_helper(&self.root, target, k, 0, &mut heap);

            let res: Vec<T> = heap.into_sorted_vec().iter().map(|x| x.second).collect();
            Rc::new(res)
        });
        Rc::clone(&self.cache.borrow()[&target])
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::RGB;
    #[test]
    #[should_panic]
    fn empty_tree_panic() {
        let tree: KdTree<RGB, 3> = KdTree::new(Vec::new());
        tree.k_nn(RGB::default(), 1);
    }
    #[test]
    fn empty_tree() {
        let tree: KdTree<RGB, 3> = KdTree::new(Vec::new());
        assert!(tree.k_nn(RGB::default(), 0).is_empty());
    }
    #[test]
    fn regular1() {
        let palette = vec![
            RGB { r: 0, g: 0, b: 255 },
            RGB { r: 0, g: 255, b: 0 },
            RGB { r: 255, g: 0, b: 0 },
            RGB {
                r: 10,
                g: 10,
                b: 10,
            },
        ];
        let tree = KdTree::new(palette);
        let res = tree.k_nn(RGB { r: 30, g: 0, b: 0 }, 3);
        assert!(
            res[0]
                == RGB {
                    r: 10,
                    g: 10,
                    b: 10
                }
        );
        assert!(res[1] == RGB { r: 255, g: 0, b: 0 });
        assert!(res.len() == 3);
    }
    #[test]
    fn regular2() {
        let palette = vec![
            RGB { r: 2, g: 1, b: 1 },
            RGB { r: 0, g: 3, b: 2 },
            RGB { r: 2, g: 2, b: 4 },
            RGB { r: 5, g: 0, b: 0 },
        ];
        let tree = KdTree::new(palette);
        let res = tree.k_nn(RGB { r: 1, g: 1, b: 1 }, 3);
        assert!(res[0] == RGB { r: 2, g: 1, b: 1 });
        assert!(res[1] == RGB { r: 0, g: 3, b: 2 });
        assert!(res[2] == RGB { r: 2, g: 2, b: 4 });
    }
}
