use std::mem::{replace, ManuallyDrop, MaybeUninit};

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

    unsafe fn inner(self) -> Union<T, E> {
        replace(&mut ManuallyDrop::new(self).data, MaybeUninit::uninit()).assume_init()
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

    #[inline(never)]
    pub fn map_or_else<U, D: FnOnce(E) -> U, F: FnOnce(T) -> U>(self, g: D, f: F) -> U {
        struct SafeFn<T, E, U, D, F> {
            checksum: usize,
            function: unsafe fn(HardResult<T, E>, D, F) -> U,
        }

        impl<T, E, U, D, F> SafeFn<T, E, U, D, F> {
            fn seal(&mut self) -> &Self {
                self.checksum = (self as *const Self as usize) ^ (self.function as usize);
                self
            }
        }

        let mut then_do = SafeFn {
            checksum: 0,
            function: |this: HardResult<T, E>, _: D, f: F| unsafe { f(this.unwrap_unchecked()) },
        };
        let mut else_do = SafeFn {
            checksum: 0,
            function: |this: HardResult<T, E>, g: D, _: F| unsafe {
                g(this.unwrap_err_unchecked())
            },
        };

        let mask_0 = self.tag ^ S_FST;
        let mask_1 = self.tag ^ S_SND;

        let address_0 = then_do.seal() as *const _ as usize & mask_1;
        let address_1 = else_do.seal() as *const _ as usize & mask_0;
        let address = address_0 ^ address_1;

        unsafe {
            let ptr = address as *const SafeFn<T, E, U, D, F>;
            if (*ptr).checksum ^ ptr as usize != (*ptr).function as usize {
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
impl std::ops::BitOr for HardResult<(), ()> {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        let mask = !self.tag ^ other.tag;
        unsafe { Self::from_tag(self.tag & mask | S_FST & !mask) }
    }
}

impl std::ops::BitAnd for HardResult<(), ()> {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        let mask = !self.tag ^ other.tag;
        unsafe { Self::from_tag(self.tag & mask | S_SND & !mask) }
    }
}

impl std::ops::BitXor for HardResult<(), ()> {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        let mask = !self.tag ^ other.tag;
        unsafe { Self::from_tag(mask & S_SND | !mask & S_FST) }
    }
}

impl std::ops::Not for HardResult<(), ()> {
    type Output = Self;

    fn not(self) -> Self {
        unsafe { Self::from_tag(!self.tag) }
    }
}

impl HardResult<(), ()> {
    /// # Safety
    /// tag must be a valid tag value
    unsafe fn from_tag(tag: usize) -> Self {
        const EMPTY_DATA: MaybeUninit<Union<(), ()>> = MaybeUninit::new(Union {
            fst: ManuallyDrop::new(()),
        });

        HardResult {
            tag,
            data: EMPTY_DATA,
        }
    }
}

impl<T, E> HardResult<T, E> {
    pub fn truncate(&self) -> HardResult<(), ()> {
        unsafe { HardResult::from_tag(self.tag) }
    }
}
