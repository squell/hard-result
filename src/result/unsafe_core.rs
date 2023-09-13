use std::mem::{forget, replace, transmute, ManuallyDrop, MaybeUninit};

type InternalRepresentation = usize;

union Union<A, B> {
    fst: ManuallyDrop<A>,
    snd: ManuallyDrop<B>,
}

pub struct HardResult<A, B> {
    tag: InternalRepresentation,
    data: MaybeUninit<Union<A, B>>,
}

const S_FST: InternalRepresentation = 0xAAAAAAAAAAAAAAAA;
const S_SND: InternalRepresentation = 0x5555555555555555;

impl<T, E> HardResult<T, E> {
    pub const fn new(value: T) -> Self {
        Self {
            tag: S_FST,
            data: MaybeUninit::new(Union {
                fst: ManuallyDrop::new(value),
            }),
        }
    }

    pub const fn new_err(value: E) -> Self {
        Self {
            tag: S_SND,
            data: MaybeUninit::new(Union {
                snd: ManuallyDrop::new(value),
            }),
        }
    }

    unsafe fn inner(mut self) -> Union<T, E> {
        let result = replace(&mut self.data, MaybeUninit::uninit()).assume_init();
        forget(self);

        result
    }

    //NOTE: T and E are not dynamically sized, so a &T and &E have the same representation
    pub const fn as_ref(&self) -> HardResult<&T, &E> {
        let ptr = self.data.as_ptr() as *const T;
        HardResult {
            tag: self.tag,
            data: MaybeUninit::new(Union {
                fst: ManuallyDrop::new(unsafe { &*ptr }),
            }),
        }
    }

    pub fn as_mut(&mut self) -> HardResult<&mut T, &mut E> {
        let ptr = self.data.as_mut_ptr() as *mut T;
        HardResult {
            tag: self.tag,
            data: MaybeUninit::new(Union {
                fst: ManuallyDrop::new(unsafe { &mut *ptr }),
            }),
        }
    }

    pub fn map_or_else<U, D: FnOnce(E) -> U, F: FnOnce(T) -> U>(self, g: D, f: F) -> U {
        unsafe fn then_do<T, E, U>(
            this: HardResult<T, E>,
            _g: impl FnOnce(E) -> U,
            f: impl FnOnce(T) -> U,
        ) -> U {
            f(ManuallyDrop::into_inner(this.inner().fst))
        }

        unsafe fn else_do<T, E, U>(
            this: HardResult<T, E>,
            g: impl FnOnce(E) -> U,
            _f: impl FnOnce(T) -> U,
        ) -> U {
            g(ManuallyDrop::into_inner(this.inner().snd))
        }

        type BodyFunction<T, E, U, D, F> = unsafe fn(HardResult<T, E>, D, F) -> U;

        let mask_0 = self.tag ^ S_FST;
        let mask_1 = self.tag ^ S_SND;

        #[allow(clippy::transmutes_expressible_as_ptr_casts)]
        unsafe {
            let address_0 =
                transmute::<BodyFunction<T, E, U, D, F>, InternalRepresentation>(then_do) & mask_1;
            let address_1 =
                transmute::<BodyFunction<T, E, U, D, F>, InternalRepresentation>(else_do) & mask_0;

            transmute::<InternalRepresentation, BodyFunction<T, E, U, D, F>>(address_0 ^ address_1)(
                self, g, f,
            )
        }
    }

    /// # Safety
    /// You must be certain that this HardResult contains a 'Ok' value
    pub unsafe fn unwrap_unchecked(self) -> T {
        ManuallyDrop::into_inner(self.inner().fst)
    }

    /// # Safety
    /// You must be certain that this HardResult contains a 'Err' value
    pub unsafe fn unwrap_err_unchecked(self) -> E {
        ManuallyDrop::into_inner(self.inner().snd)
    }
}

impl<T, E> Drop for HardResult<T, E> {
    fn drop(&mut self) {
        HardResult {
            tag: self.tag,
            data: replace(&mut self.data, MaybeUninit::uninit()),
        }
        .map_or_else(|_| {}, |_| {})
    }
}

/// Boolean primitives that are just handy to have
const EMPTY_DATA: MaybeUninit<Union<(), ()>> = MaybeUninit::new(Union {
    fst: ManuallyDrop::new(()),
});

impl std::ops::BitOr for HardResult<(), ()> {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        let mask = !self.tag ^ other.tag;
        Self {
            tag: self.tag & mask | S_FST & !mask,
            data: EMPTY_DATA,
        }
    }
}

impl std::ops::BitAnd for HardResult<(), ()> {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        let mask = !self.tag ^ other.tag;
        Self {
            tag: self.tag & mask | S_SND & !mask,
            data: EMPTY_DATA,
        }
    }
}

impl std::ops::BitXor for HardResult<(), ()> {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        let mask = !self.tag ^ other.tag;
        Self {
            tag: mask & S_SND | !mask & S_FST,
            data: EMPTY_DATA,
        }
    }
}

impl std::ops::Not for HardResult<(), ()> {
    type Output = Self;

    fn not(self) -> Self {
        Self {
            tag: !self.tag,
            data: EMPTY_DATA,
        }
    }
}
