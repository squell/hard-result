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

    pub fn r#if_else(self, then_do: impl FnOnce(), else_do: impl FnOnce()) {
        self.map_or_else(|()| else_do(), |()| then_do())
    }

    fn repeat(mut test: impl FnMut() -> Self) {
        test().if_else(|| Self::repeat(test), || ())
    }

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
