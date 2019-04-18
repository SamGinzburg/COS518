use crate::rand::prelude::SliceRandom;

#[derive(Debug, Eq, PartialEq)]
pub struct Permutation {
    map: Vec<usize>,
}

impl Permutation {
    pub fn sample(m : usize) -> Permutation {
        let mut map : Vec<usize> = (1..m).collect();
        map.shuffle(&mut rand::thread_rng());
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

    pub fn apply_inverse<T>(&self, input : Vec<T>) -> Vec<T> {
        let mut tmp : Vec<Option<T>> = Vec::with_capacity(input.len());
        for x in input {
            tmp.push(Some(x));
        }

        let mut output : Vec<T> = Vec::with_capacity(tmp.len());
        for i in 0..tmp.len() {
            tmp.push(None);
            output.push(tmp.swap_remove(*self.map.get(i).unwrap()).unwrap());
        }

        output
    }

    #[cfg(test)]
    fn from_vec(map : Vec<usize>) -> Permutation {
        Permutation { map }
    }
}

#[cfg(test)]
mod text {
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
        let pi = Permutation::from_vec(vec![1, 0, 2]);
        let v = vec!['a', 'b', 'c'];
        let w = pi.apply_inverse(pi.apply(v));

        assert_eq!(w, vec!['a', 'b', 'c']);
    }

    #[test]
    fn sampling_random() {
        let pi = Permutation::sample(100);
        let rho = Permutation::sample(100);

        assert_ne!(pi, rho);
    }
}
