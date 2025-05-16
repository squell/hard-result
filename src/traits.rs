/// Trait implementations
use super::HardResult;

use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

impl<T: Clone, E: Clone> Clone for HardResult<T, E> {
    fn clone(&self) -> Self {
        self.as_ref().map_or_else(
            |e| HardResult::new_err(e.clone()),
            |x| HardResult::new(x.clone()),
        )
    }
}

impl<T: Debug, E: Debug> Debug for HardResult<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.as_ref()
            .map(|x| {
                write!(f, "HardResult::new(")?;
                x.fmt(f)?;
                write!(f, ")")?;
                Ok(())
            })
            .unwrap_or_else(|e| {
                write!(f, "HardResult::new_err(")?;
                e.fmt(f)?;
                write!(f, ")")?;
                Ok(())
            })
    }
}

impl<T: Hash, E: Hash> Hash for HardResult<T, E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref()
            .map(|x| x.hash(state))
            .unwrap_or_else(|e| e.hash(state))
    }
}

#[cfg(feature = "try")]
impl<T, E> std::ops::FromResidual<HardResult<std::convert::Infallible, E>> for HardResult<T, E> {
    fn from_residual(residual: HardResult<std::convert::Infallible, E>) -> HardResult<T, E> {
        residual.map_or_else(
            |e| HardResult::new_err(e),
            |_| panic!("infallible value encountered"),
        )
    }
}

#[cfg(feature = "try")]
impl<T, E> std::ops::Try for HardResult<T, E> {
    type Output = T;
    type Residual = HardResult<std::convert::Infallible, E>;

    fn from_output(output: Self::Output) -> Self {
        HardResult::new(output)
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        use std::ops::ControlFlow;
        self.map_or_else(
            |e| ControlFlow::Break(HardResult::new_err(e)),
            |v| ControlFlow::Continue(v),
        )
    }
}
