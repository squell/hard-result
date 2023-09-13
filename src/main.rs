use std::mem::{transmute, ManuallyDrop};

type InternalRepresentation = usize;

union Union<A, B> {
    fst: ManuallyDrop<A>,
    snd: ManuallyDrop<B>,
}

struct HardResult<A, B> {
    tag: InternalRepresentation,
    data: Option<Union<A, B>>,
}

struct HardOption<A>(HardResult<A, ()>);

struct HardBool(HardResult<(), ()>);

const S_FST: InternalRepresentation = 0xAAAAAAAAAAAAAAAA;
const S_SND: InternalRepresentation = 0x5555555555555555;

impl<T, E> HardResult<T, E> {
    pub fn new(value: T) -> Self {
        Self {
            tag: S_FST,
            data: Some(Union {
                fst: ManuallyDrop::new(value),
            }),
        }
    }

    pub fn new_err(value: E) -> Self {
        Self {
            tag: S_SND,
            data: Some(Union {
                snd: ManuallyDrop::new(value),
            }),
        }
    }

    fn if_then_else<U, D: FnOnce(E) -> U, F: FnOnce(T) -> U>(&mut self, g: D, f: F) -> U {
        unsafe fn then_do<T, E, U>(
            this: &mut HardResult<T, E>,
            _g: impl FnOnce(E) -> U,
            f: impl FnOnce(T) -> U,
        ) -> U {
            f(ManuallyDrop::into_inner(this.data.take().unwrap().fst))
        }

        unsafe fn else_do<T, E, U>(
            this: &mut HardResult<T, E>,
            g: impl FnOnce(E) -> U,
            _f: impl FnOnce(T) -> U,
        ) -> U {
            g(ManuallyDrop::into_inner(this.data.take().unwrap().snd))
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
        self.if_then_else(g, f)
    }

    pub fn unwrap(self) -> T {
        self.map_or_else(|_| panic!("unwrap on HardResult"), |x| x)
    }

    pub fn unwrap_err(self) -> E {
        self.map_or_else(|x| x, |_| panic!("unwrap on HardResult"))
    }

    pub unsafe fn unwrap_unchecked(mut self) -> T {
        ManuallyDrop::into_inner(self.data.take().unwrap_unchecked().fst)
    }

    pub unsafe fn unwrap_err_unchecked(mut self) -> E {
        ManuallyDrop::into_inner(self.data.take().unwrap_unchecked().snd)
    }
}

impl<T, E> Drop for HardResult<T, E> {
    fn drop(&mut self) {
        if self.data.is_some() {
            self.if_then_else(|_x| {}, |_x| {})
        }
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
}
