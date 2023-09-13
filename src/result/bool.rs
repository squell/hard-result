use super::{HardBool, HardOption};

impl HardBool {
    pub const fn r#true() -> Self {
        Self::new(())
    }

    pub const fn r#false() -> Self {
        Self::new_err(())
    }

    pub fn then<T>(self, f: impl FnOnce() -> T) -> HardOption<T> {
        self.map(|()| f())
    }

    pub fn then_some<T>(self, value: T) -> HardOption<T> {
        self.map(|()| value)
    }

    pub fn if_else<U>(self, then_do: impl FnOnce() -> U, else_do: impl FnOnce() -> U) -> U {
        self.map_or_else(|()| else_do(), |()| then_do())
    }

    pub fn r#if<U>(self, then_do: impl FnOnce() -> U) -> Else<U> {
        Else(self.map(|()| then_do()))
    }

    #[cfg(feature = "loops")]
    fn repeat(mut test: impl FnMut() -> Self) {
        test().if_else(|| Self::repeat(test), || ())
    }

    #[cfg(feature = "loops")]
    pub fn r#while(mut test: impl FnMut() -> Self, mut body: impl FnMut()) {
        test().if_else(
            || {
                Self::repeat(|| {
                    body();
                    test()
                })
            },
            || (),
        );
    }
}

pub struct Else<U>(HardOption<U>);

impl<U> Else<U> {
    pub fn r#else(self, f: impl FnOnce() -> U) -> U {
        self.0.unwrap_or_else(|()| f())
    }
}

impl<B: std::convert::Into<bool>> From<B> for HardBool {
    fn from(value: B) -> Self {
        if value.into() {
            HardBool::r#true()
        } else {
            HardBool::r#false()
        }
    }
}
