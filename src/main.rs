use std::mem::{forget, replace, transmute, ManuallyDrop, MaybeUninit};

type InternalRepresentation = usize;

union Union<A, B> {
    fst: ManuallyDrop<A>,
    snd: ManuallyDrop<B>,
}

struct HardResult<A, B> {
    tag: InternalRepresentation,
    data: MaybeUninit<Union<A, B>>,
}

struct HardOption<A>(HardResult<A, ()>);

struct HardBool(HardResult<(), ()>);

const S_FST: InternalRepresentation = 0xAAAAAAAAAAAAAAAA;
const S_SND: InternalRepresentation = 0x5555555555555555;

impl<T, E> HardResult<T, E> {
    pub fn new(value: T) -> Self {
        Self {
            tag: S_FST,
            data: MaybeUninit::new(Union {
                fst: ManuallyDrop::new(value),
            }),
        }
    }

    pub fn new_err(value: E) -> Self {
        Self {
            tag: S_SND,
            data: MaybeUninit::new(Union {
                snd: ManuallyDrop::new(value),
            }),
        }
    }

    unsafe fn inner(&mut self) -> Union<T, E> {
        replace(&mut self.data, MaybeUninit::uninit()).assume_init()
    }

    fn if_then_else<U, D: FnOnce(E) -> U, F: FnOnce(T) -> U>(&mut self, g: D, f: F) -> U {
        unsafe fn then_do<T, E, U>(
            this: &mut HardResult<T, E>,
            _g: impl FnOnce(E) -> U,
            f: impl FnOnce(T) -> U,
        ) -> U {
            f(ManuallyDrop::into_inner(this.inner().fst))
        }

        unsafe fn else_do<T, E, U>(
            this: &mut HardResult<T, E>,
            g: impl FnOnce(E) -> U,
            _f: impl FnOnce(T) -> U,
        ) -> U {
            g(ManuallyDrop::into_inner(this.inner().snd))
        }

        type BodyFunction<T, E, U, D, F> = unsafe fn(&'_ mut HardResult<T, E>, D, F) -> U;

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

    pub fn map_or_else<U, D: FnOnce(E) -> U, F: FnOnce(T) -> U>(mut self, g: D, f: F) -> U {
        let result = self.if_then_else(g, f);
        forget(self);

        result
    }

    pub fn unwrap(self) -> T {
        self.map_or_else(|_| panic!("unwrap on HardResult"), |x| x)
    }

    pub fn unwrap_err(self) -> E {
        self.map_or_else(|x| x, |_| panic!("unwrap on HardResult"))
    }

    pub unsafe fn unwrap_unchecked(mut self) -> T {
        ManuallyDrop::into_inner(self.inner().fst)
    }

    pub unsafe fn unwrap_err_unchecked(mut self) -> E {
        ManuallyDrop::into_inner(self.inner().snd)
    }
}

impl<T, E> Drop for HardResult<T, E> {
    fn drop(&mut self) {
        self.if_then_else(|_x| {}, |_x| {})
    }
}

struct Dummy;

impl Drop for Dummy {
    fn drop(&mut self) {
        println!("Mmm mmm");
    }
}

fn main() {
    let foo = HardResult::<&'static str, i32>::new("erawr");

    foo.map_or_else(
        |x| {
            println!("erro value {x}");
        },
        |x| {
            println!("OK value {x}");
        },
    );

    let bar = HardResult::<Dummy, ()>::new(Dummy);
    bar.unwrap();
}
