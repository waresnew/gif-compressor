use std::{cmp::Ordering, collections::BinaryHeap};

use crate::{image::Rgb, nearest_neighbour::NnSolver};

type MaybeNode = Option<Box<Node>>;

#[derive(Debug)]
struct Node {
    val: Rgb,
    left: MaybeNode,
    right: MaybeNode,
}

#[derive(Debug)]
///dynamic insert/delete not implemented
pub struct KdTree {
    root: MaybeNode,
    size: usize,
}

impl KdTree {
    fn make_subtree(lst: &mut [Rgb], depth: usize) -> MaybeNode {
        if lst.is_empty() {
            return None;
        }
        let dim = depth % 3;
        let mid = lst.len() / 2;
        let split = lst.select_nth_unstable_by(mid, |a, b| a.get(dim).cmp(&b.get(dim)));
        Some(Box::new(Node {
            val: *split.1,
            left: Self::make_subtree(split.0, depth + 1),
            right: Self::make_subtree(split.2, depth + 1),
        }))
    }
    fn k_nn_helper(
        cur: &MaybeNode,
        target: Rgb,
        k: usize,
        depth: usize,
        ans: &mut BinaryHeap<(u32, Rgb)>,
    ) {
        let Some(cur) = cur else {
            return;
        };
        let dim = depth % 3;
        let mut went_left = false;
        let dis = target.distance_sq(cur.val);
        ans.push((dis, cur.val));
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
        let max_dis = ans.peek().unwrap().0;
        if ans.len() < k || max_dis > (target.get(dim) - cur.val.get(dim)).unsigned_abs().pow(2) {
            Self::k_nn_helper(
                if went_left { &cur.right } else { &cur.left },
                target,
                k,
                depth + 1,
                ans,
            );
        }
    }
    pub fn k_nn(&self, target: Rgb, k: usize) -> Vec<Rgb> {
        assert!(k <= self.size);
        if k > self.size {
            panic!("k>self.size: k={}, size={}", k, self.size);
        }
        let mut heap: BinaryHeap<(u32, Rgb)> = BinaryHeap::with_capacity(k);
        Self::k_nn_helper(&self.root, target, k, 0, &mut heap);
        heap.into_sorted_vec().iter().map(|x| x.1).collect()
    }
}
impl NnSolver for KdTree {
    fn new(mut lst: Vec<Rgb>) -> Self {
        Self {
            size: lst.len(),
            root: Self::make_subtree(&mut lst, 0),
        }
    }
    fn nn(&self, target: Rgb, exclude: Option<[Rgb; 2]>) -> Option<Rgb> {
        let exclude_len = if exclude.is_some() { 2 } else { 0 };
        let res = self.k_nn(target, exclude_len + 1);
        res.into_iter().find(|&x| {
            if let Some(exclude) = exclude {
                x != exclude[0] && x != exclude[1]
            } else {
                true
            }
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::Rgb;
    #[test]
    #[should_panic]
    fn empty_tree_panic() {
        let tree: KdTree = KdTree::new(Vec::new());
        tree.k_nn(Rgb::default(), 1);
    }
    #[test]
    fn empty_tree() {
        let tree: KdTree = KdTree::new(Vec::new());
        assert!(tree.k_nn(Rgb::default(), 0).is_empty());
    }
    #[test]
    fn regular1() {
        let palette = vec![
            Rgb::new(0, 0, 255),
            Rgb::new(0, 255, 0),
            Rgb::new(255, 0, 0),
            Rgb::new(10, 10, 10),
        ];
        let tree = KdTree::new(palette);
        let res = tree.k_nn(Rgb::new(30, 0, 0), 3);
        assert!(res[0] == Rgb::new(10, 10, 10));
        assert!(res[1] == Rgb::new(255, 0, 0));
        assert!(res.len() == 3);
    }
    #[test]
    fn regular2() {
        let palette = vec![
            Rgb::new(2, 1, 1),
            Rgb::new(0, 3, 2),
            Rgb::new(2, 2, 4),
            Rgb::new(5, 0, 0),
        ];
        let tree = KdTree::new(palette);
        let res = tree.k_nn(Rgb::new(1, 1, 1), 3);
        assert!(res[0] == Rgb::new(2, 1, 1));
        assert!(res[1] == Rgb::new(0, 3, 2));
        assert!(res[2] == Rgb::new(2, 2, 4));
    }
}
