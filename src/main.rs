mod result;
pub use result::{HardBool, HardOption, HardResult};

struct Dummy;

impl Drop for Dummy {
    fn drop(&mut self) {
        println!("Mmm mmm");
    }
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

    obj.ok().is_some().r#if(|| println!("It is a some!"), || {});

    let bar = HardResult::<Dummy, ()>::new(Dummy);
    bar.unwrap();
}
