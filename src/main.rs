#![cfg_attr(feature = "try", feature(try_trait_v2))]

mod result;
pub use result::{HardBool, HardOption, HardResult, FALSE, TRUE};

struct Dummy;

impl Drop for Dummy {
    fn drop(&mut self) {
        println!("Mmm mmm");
    }
}

#[cfg(feature = "try")]
fn pass(y: i32) -> HardOption<i32> {
    fn test(x: i32) -> HardOption<i32> {
        if x == 0 {
            HardOption::r#none()
        } else {
            HardOption::r#some(x)
        }
    }

    let x = test(y)?;

    HardOption::r#some(x + 1)
}

fn main() {
    let mut obj = HardResult::<&'static str, i32>::new("erawr");
    *obj.as_mut().unwrap() = "a new string";

    obj.as_ref().map_or_else(
        |x| {
            println!("erro value {x}");
        },
        |x| {
            println!("OK value {x}");
        },
    );

    obj.ok()
        .is_some()
        .r#if(|| println!("It is a some!"))
        .r#else(|| println!("It is none!"));

    let bar = HardResult::<Dummy, ()>::new(Dummy);
    bar.unwrap();

    #[cfg(feature = "try")]
    println!("{}", pass(41).unwrap());

    println!("{:?}", TRUE);

    use std::cell::Cell;
    let x = Cell::new(1000000);
    harder! {
	while ({x.get() > 0}.into()) {
	    println!("{} bottles of beer!", x.get());
	    x.replace(x.get() - 1);
	}
    }
}
