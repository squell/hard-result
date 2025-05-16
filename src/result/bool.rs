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
        #[inline(never)]
        fn trampoline<U>(status: &HardBool, g: fn(()) -> U, f: fn(()) -> U) -> U {
            status.clone().map_or_else(g, f)
        }

        f(status);

        trampoline::<fn(_, _)>(status, |()| |_, _| (), |()| Self::repeat)(status, f);
    }

    //NOTE: This relies on tail-call optimization, which is not guaranteed!
    pub fn r#while(mut test: impl FnMut() -> Self, mut body: impl FnMut()) {
        let mut status = test();
        Self::repeat(&mut status, &mut |cond: &mut HardBool| {
            body();
            *cond = test();
        });
    }

    pub fn r#do_while(mut body: impl FnMut() -> Self) {
        let mut status = HardBool::r#true();
        Self::repeat(&mut status, &mut |cond: &mut HardBool| {
            *cond = body();
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
