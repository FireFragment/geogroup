use geogroup_algo::*;

fn main() {
    println!(
        "{}",
        algo::sort(vec![1, 2, 1, 10, 5, 2, 1, 2, 1], 128).print_tree()
    );
}
