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

    fn repeat(status: &mut Self, f: &mut dyn FnMut(&mut Self)) {
        fn trampoline<U>(status: &HardBool, g: fn() -> U, f: fn() -> U) -> U {
            status.clone().map_or_else(|()| g(), |()| f())
        }

        f(status);

        trampoline(
            status,
            || (|_, _| {}) as fn(&mut Self, &mut dyn FnMut(&mut Self)),
            || Self::repeat,
        )(status, f);
    }

    pub fn r#while(mut test: impl FnMut() -> Self, mut body: impl FnMut()) {
        let mut status = test();
        Self::repeat(&mut status, &mut |cond: &mut HardBool| {
            body();
            *cond = test();
        });
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
