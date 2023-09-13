use super::{HardBool, HardOption, HardResult};

// Misc methods
impl<T> HardOption<T> {
    pub const fn r#some(value: T) -> HardOption<T> {
        Self::new(value)
    }

    pub const fn r#none() -> HardOption<T> {
        Self::new_err(())
    }

    pub fn filter(self, pred: impl FnOnce(&T) -> HardBool) -> HardOption<T> {
        self.and_then(|x| pred(&x).map(|()| x))
    }
}

impl<T> HardOption<HardOption<T>> {
    pub fn flatten(self) -> HardOption<T> {
        self.map_or_else(|()| HardOption::new_err(()), |x| x)
    }
}

// Inserters and updaters
impl<T> HardOption<T> {
    pub fn insert(&mut self, value: T) -> &mut T {
        *self = HardOption::new(value);

        self.as_mut().unwrap()
    }

    pub fn take(&mut self) -> HardOption<T> {
        let mut empty = HardOption::new_err(());
        std::mem::swap(&mut empty, self);

        empty
    }

    pub fn replace(&mut self, value: T) -> HardOption<T> {
        let old = self.take();
        *self = HardOption::new(value);

        old
    }

    pub fn get_or_insert_with(&mut self, value: impl FnOnce() -> T) -> &mut T {
        let local = self.take().unwrap_or_else(|_| value());

        self.insert(local)
    }

    pub fn get_or_insert(&mut self, value: T) -> &mut T {
        self.get_or_insert_with(|| value)
    }
}

impl<T: Default> HardOption<T> {
    pub fn get_or_insert_default(&mut self) -> &mut T {
        self.get_or_insert_with(T::default)
    }
}

// Result converters
impl<T> HardOption<T> {
    pub fn ok_or_else<E>(self, error: impl FnOnce() -> E) -> HardResult<T, E> {
        self.map_err(|()| error())
    }

    pub fn ok_or<E>(self, error: E) -> HardResult<T, E> {
        self.map_err(|()| error)
    }

    pub fn is_some(&self) -> HardBool {
        self.is_ok()
    }

    pub fn is_none(&self) -> HardBool {
        self.is_err()
    }
}
