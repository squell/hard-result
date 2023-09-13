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

    fn map_or_else<U, D: FnOnce(E) -> U, F: FnOnce(T) -> U>(self, g: D, f: F) -> U {
        unsafe fn then_do<T, E, U>(
            mut this: HardResult<T, E>,
            _g: impl FnOnce(E) -> U,
            f: impl FnOnce(T) -> U,
        ) -> U {
            f(ManuallyDrop::into_inner(this.data.take().unwrap().fst))
        }

        unsafe fn else_do<T, E, U>(
            mut this: HardResult<T, E>,
            g: impl FnOnce(E) -> U,
            _f: impl FnOnce(T) -> U,
        ) -> U {
            g(ManuallyDrop::into_inner(this.data.take().unwrap().snd))
        }

        type BodyFunction<T, E, U, D, F> = unsafe fn(HardResult<T, E>, D, F) -> U;

        let mask_0 = self.tag ^ S_FST;
        let mask_1 = self.tag ^ S_SND;

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
}

impl<T, E> Drop for HardResult<T, E> {
    fn drop(&mut self) {
        //self.map_or_else(|_x| { }, |_x| { })
    }
}

fn main() {
    let foo = HardResult::<&str, i32>::new("erawr");

    foo.map_or_else(
        |x| {
            println!("erro value {x}");
        },
        |x| {
            println!("OK value {x}");
        },
    )
}
