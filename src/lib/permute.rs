use crate::rand::prelude::SliceRandom;

use std::cmp::Ordering;
use std::iter;

#[derive(Debug, Eq, PartialEq)]
pub struct Permutation {
    map: Vec<usize>,
}

impl Permutation {
    pub fn sample(m : usize) -> Permutation {
        if m == 0 {
            // todo: do we need this?
            return Permutation { map: vec![] }
        }
        let mut map : Vec<usize> = (0..m).collect();
        map.shuffle(&mut rand::thread_rng());
        Permutation { map }
    }

    pub fn inverse(&self) -> Permutation {
        let mut map : Vec<usize> = iter::repeat(0).take(self.map.len()).collect();
        for (i,j) in self.map.iter().enumerate() {
            map[*j] = i;
        }
        Permutation { map }
    }

    pub fn from_sort<T, F>(v : &mut Vec<T>, f : F) -> Permutation
    where F: Fn(&T, &T) -> Ordering {
        let n = v.len();

        let mut indexed : Vec<(usize, T)> = Vec::with_capacity(n);
        for (i,t) in v.drain(..).enumerate() {
            indexed.push((i,t));
        }

        indexed.sort_by(|(_i1,t1), (_i2,t2)| f(t1,t2));

        let mut map : Vec<usize> = Vec::with_capacity(n);
        for (i,t) in indexed {
            v.push(t);
            map.push(i);
        }

        Permutation { map }
    }

    pub fn apply<T>(&self, input : Vec<T>) -> Vec<T> {
        let mut tmp : Vec<Option<T>> = Vec::with_capacity(input.len());
        for x in input {
            tmp.push(Some(x));
        }

        let mut output : Vec<T> = Vec::with_capacity(tmp.len());
        for i in &self.map {
            tmp.push(None);
            output.push(tmp.swap_remove(*i).unwrap());
        }

        output
    }

    #[cfg(test)]
    fn from_vec(map : Vec<usize>) -> Permutation {
        Permutation { map }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn apply_correct() {
        let pi = Permutation::from_vec(vec![1, 0, 2]);
        let v = vec!['a', 'b', 'c'];
        let w = pi.apply(v);

        assert_eq!(w, vec!['b', 'a', 'c']);
    }

    #[test]
    fn invert_correct() {
        let pi = Permutation::from_vec(vec![2, 0, 1]);
        let v = vec!['a', 'b', 'c'];
        let w = pi.inverse().apply(pi.apply(v));

        assert_eq!(w, vec!['a', 'b', 'c']);
    }

    #[test]
    fn sampling_random() {
        let pi = Permutation::sample(100);
        let rho = Permutation::sample(100);

        assert_ne!(pi, rho);
    }

    #[test]
    fn from_sort_correct() {
        let v = vec![1,8,2,9,3,5,8,1,3];
        let mut v_clone = v.clone();
        let pi = Permutation::from_sort(&mut v_clone, Ord::cmp);

        assert_eq!(pi.apply(v.clone()), v_clone);
    }
}
