use geogroup_algo::*;

fn main() {
    let r = algo::sort_to_binary_tree(vec![1, 2, 1, 10, 5, 2, 1, 2, 1]);

    println!("{}", algo::flatten(r, 128).print_tree());
}
