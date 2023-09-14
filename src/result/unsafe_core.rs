use std::mem::{forget, replace, ManuallyDrop, MaybeUninit};

union Union<A, B> {
    fst: ManuallyDrop<A>,
    snd: ManuallyDrop<B>,
}

pub struct HardResult<A, B> {
    tag: usize,
    data: MaybeUninit<Union<A, B>>,
}

const S_FST: usize = 0xAAAAAAAAAAAAAAAA;
const S_SND: usize = 0x5555555555555555;

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
    pub fn as_ref(&self) -> HardResult<&T, &E> {
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
        struct SafeFn<T, E, U, D, F> {
            redundancy: *const Self,
            function: unsafe fn(HardResult<T, E>, D, F) -> U,
        }

        impl<T, E, U, D, F> SafeFn<T, E, U, D, F> {
            fn seal(&mut self) -> &Self {
                self.redundancy = self;
                self
            }
        }

        let mut then_do = SafeFn {
            redundancy: std::ptr::null(),
            function: |this: HardResult<T, E>, _: D, f: F| unsafe {
                f(ManuallyDrop::into_inner(this.inner().fst))
            },
        };
        let mut else_do = SafeFn {
            redundancy: std::ptr::null(),
            function: |this: HardResult<T, E>, g: D, _: F| unsafe {
                g(ManuallyDrop::into_inner(this.inner().snd))
            },
        };

        let mask_0 = self.tag ^ S_FST;
        let mask_1 = self.tag ^ S_SND;

        let address_0 = then_do.seal() as *const _ as usize & mask_1;
        let address_1 = else_do.seal() as *const _ as usize & mask_0;
        let address = address_0 ^ address_1;

        unsafe {
            let ptr = address as *const SafeFn<T, E, U, D, F>;
            if (*ptr).redundancy != ptr {
                std::process::abort()
            }
            ((*ptr).function)(self, g, f)
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

impl<T, E> HardResult<T, E> {
    pub fn truncate(&self) -> HardResult<(), ()> {
        HardResult {
            tag: self.tag,
            data: EMPTY_DATA,
        }
    }
}
