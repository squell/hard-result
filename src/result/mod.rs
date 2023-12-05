mod bool;
mod macros;
mod option;
mod traits;

mod unsafe_core;

pub use unsafe_core::HardResult;

pub type HardOption<A> = HardResult<A, ()>;
pub type HardBool = HardOption<()>;

pub const r#TRUE: HardBool = HardBool::r#true();
pub const r#FALSE: HardBool = HardBool::r#false();

// All the unwraps
impl<T, E> HardResult<T, E> {
    pub fn unwrap(self) -> T {
        self.map_or_else(|_| panic!("unwrap on Err"), |x| x)
    }

    pub fn unwrap_err(self) -> E {
        self.map_or_else(|x| x, |_| panic!("unwrap_err on Ok"))
    }

    pub fn unwrap_or(self, default: T) -> T {
        self.map_or_else(|_| default, |x| x)
    }

    pub fn unwrap_or_else(self, f: impl FnOnce(E) -> T) -> T {
        self.map_or_else(f, |x| x)
    }
}

impl<T: Default, E> HardResult<T, E> {
    #[allow(clippy::unwrap_or_default)]
    pub fn unwrap_or_default(self) -> T {
        self.unwrap_or(T::default())
    }
}

impl<T, E: std::fmt::Debug> HardResult<T, E> {
    pub fn expect(self, msg: &str) -> T {
        self.map_or_else(|e| panic!("{msg}: {e:?}"), |x| x)
    }
}

impl<T: std::fmt::Debug, E> HardResult<T, E> {
    pub fn expect_err(self, msg: &str) -> E {
        self.map_or_else(|e| e, |x| panic!("{msg}: {x:?}"))
    }
}

// All the maps
impl<T, E> HardResult<T, E> {
    pub fn map<T2>(self, f: impl FnOnce(T) -> T2) -> HardResult<T2, E> {
        self.map_or_else(|e| HardResult::new_err(e), |x| HardResult::new(f(x)))
    }

    pub fn map_err<E2>(self, f: impl FnOnce(E) -> E2) -> HardResult<T, E2> {
        self.map_or_else(|e| HardResult::new_err(f(e)), |x| HardResult::new(x))
    }

    pub fn map_or<U>(self, default: U, f: impl FnOnce(T) -> U) -> U {
        self.map_or_else(|_| default, f)
    }

    pub fn or_else<F>(self, op: impl FnOnce(E) -> HardResult<T, F>) -> HardResult<T, F> {
        self.map_or_else(op, |x| HardResult::new(x))
    }

    pub fn or<F>(self, value: HardResult<T, F>) -> HardResult<T, F> {
        self.or_else(|_| value)
    }

    pub fn and_then<U>(self, op: impl FnOnce(T) -> HardResult<U, E>) -> HardResult<U, E> {
        self.map_or_else(|e| HardResult::new_err(e), op)
    }

    pub fn and<U>(self, value: HardResult<U, E>) -> HardResult<U, E> {
        self.and_then(|_| value)
    }
}

// Option interface
impl<T, E> HardResult<T, E> {
    pub fn ok(self) -> HardOption<T> {
        self.map_err(|_| ())
    }

    pub fn err(self) -> HardOption<E> {
        self.map_or_else(|e| HardResult::new(e), |_| HardResult::new_err(()))
    }
}

impl<T, E> HardResult<HardOption<T>, E> {
    pub fn transpose(self) -> HardOption<HardResult<T, E>> {
        self.map_or_else(
            |err| HardResult::new(HardResult::new_err(err)),
            |value| value.map(|value| HardResult::new(value)),
        )
    }
}

// Clones and copies
impl<T: Copy, E> HardResult<&mut T, E> {
    pub fn copied(self) -> HardResult<T, E> {
        self.map(|x| *x)
    }
}

impl<T: Copy, E> HardResult<&T, E> {
    pub fn copied(self) -> HardResult<T, E> {
        self.map(|x| *x)
    }
}

impl<T: Clone, E> HardResult<&mut T, E> {
    pub fn cloned(self) -> HardResult<T, E> {
        self.map(|x| x.clone())
    }
}

impl<T: Clone, E> HardResult<&T, E> {
    pub fn cloned(self) -> HardResult<T, E> {
        self.map(|x| x.clone())
    }
}

// By-reference methods
impl<T: std::ops::Deref, E> HardResult<T, E> {
    pub fn as_deref(&self) -> HardResult<&T::Target, &E> {
        self.as_ref().map(|t| t.deref())
    }
}

impl<T: std::ops::DerefMut, E> HardResult<T, E> {
    pub fn as_deref_mut(&mut self) -> HardResult<&mut T::Target, &mut E> {
        self.as_mut().map(|t| t.deref_mut())
    }
}

// Boolean methods; these could be optimized by sending them to unsafe_core
impl<T, E> HardResult<T, E> {
    pub fn is_ok(&self) -> HardBool {
        self.truncate()
    }

    pub fn is_err(&self) -> HardBool {
        !self.truncate()
    }
}
