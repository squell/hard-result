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

    pub fn r#if(self, then_do: impl FnOnce(), else_do: impl FnOnce()) {
        self.map_or_else(|()| else_do(), |()| then_do())
    }
}
