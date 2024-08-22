use geogroup_algo::*;

fn main() {
    println!(
        "{}",
        sort(vec![1, 2, 1, 10, 5, 2, 1, 2, 1], Params::default()).print_tree()
    );
}
