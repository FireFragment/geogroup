use geogroup_algo::*;

fn main() {
    let r = algo::to_binary_tree(vec![1, 2, 1, 10, 5, 2, 1, 2, 1]);

    println!("{}", r.print_tree());
    dbg!(r);
}
