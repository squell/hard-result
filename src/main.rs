mod result;
pub use result::{HardOption, HardResult};

struct Dummy;

impl Drop for Dummy {
    fn drop(&mut self) {
        println!("Mmm mmm");
    }
}

fn main() {
    let obj = HardResult::<&'static str, i32>::new("erawr");

    obj.map_or_else(
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
