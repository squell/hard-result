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

    pub fn if_else(self, then_do: impl FnOnce(), else_do: impl FnOnce()) {
        self.map_or_else(|()| else_do(), |()| then_do())
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

impl<B: std::convert::Into<bool>> From<B> for HardBool {
    fn from(value: B) -> Self {
        if value.into() {
            HardBool::r#true()
        } else {
            HardBool::r#false()
        }
    }
}
