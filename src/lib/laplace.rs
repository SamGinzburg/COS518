use crate::rand::{
    distributions::{Distribution, OpenClosed01},
    Rng,
};
use std::marker::PhantomData;

/// Samples floating-point numbers according to the Laplace distribution
#[derive(Clone, Copy, Debug)]
pub struct Laplace {
    scale: f64,
    location: f64,
}

impl Laplace {
    /// Construct a new Pareto distribution with given `scale` and `location`.
    ///
    /// In the literature,
    /// `scale` is often written as b,
    /// `shape` is often written as μ.
    ///
    /// # Panics
    ///
    /// `scale` must be positive.
    pub fn new(scale: f64, location: f64) -> Laplace {
        assert!((scale >= 0.)x);
        Laplace { scale, location }
    }
}

impl Distribution<f64> for Laplace {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let u: f64 = rng.sample(OpenClosed01);

        // μ - b sgn(u) ln(1 - 2|u|)
        self.location - self.scale * u.signum() * ((-2.) * u.abs()).ln_1p()
    }
}

/// Apply arbitrary transform to distribution.
#[derive(Clone, Copy, Debug)]
pub struct TransformedDistribution<S, T, D, F> {
    parent: D,
    transform: F,
    _s: PhantomData<S>,
    _t: PhantomData<T>,
}

impl<S, T, D, F> TransformedDistribution<S, T, D, F>
where
    D: Distribution<S>,
    F: Fn(S) -> T,
{
    pub fn new(parent: D, transform: F) -> TransformedDistribution<S, T, D, F> {
        TransformedDistribution {
            parent,
            transform,
            _s: PhantomData,
            _t: PhantomData,
        }
    }
}

impl<S, T, D, F> Distribution<T> for TransformedDistribution<S, T, D, F>
where
    D: Distribution<S>,
    F: Fn(S) -> T,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        let s = rng.sample(&(self.parent));
        (self.transform)(s)
    }
}

#[cfg(test)]
mod test {
    use super::{Laplace, TransformedDistribution};
    use crate::rand::{distributions::Distribution, rngs::mock::StepRng};

    #[test]
    #[should_panic]
    fn invalid_laplace() {
        Laplace::new(-1.0, 0.0);
    }

    #[test]
    fn sample_laplace() {
        let scale = 1.0;
        let location = 2.0;
        let d = Laplace::new(scale, location);
        let mut rng = StepRng::new(0, 1);
        for _ in 0..1000 {
            d.sample(&mut rng);
        }
    }

    #[test]
    fn sample_transformed() {
        let scale = 1.0;
        let location = 2.0;
        let d = Laplace::new(scale, location);
        let e = TransformedDistribution::new(d, |x| i32::max(0, f64::ceil(x) as i32));
        let mut rng = StepRng::new(0, 1);
        for _ in 0..1000 {
            let _r = e.sample(&mut rng);
            assert!(e.sample(&mut rng) >= 0);
        }
    }
}
