use std::env;

use geocoding::{Opencage, Point, Reverse};

fn main() {
    let p = Point::new(2.12870, 41.40139);
    let oc = Opencage::new(
        env::var("OPENCAGE_API_KEY")
            .expect("Please set OpenCage API key as en environment variable OPENCAGE_API_KEY"),
    );
    let res = oc.reverse(&p);

    println!("{:?}", res.unwrap());
    // "Carrer de Calatrava, 68, 08017 Barcelona, Spain"

    println!("{:?}", oc.remaining_calls());
    // Some(2494)
}
