use geogroup_algo::*;
use geogroup_common::HiearchyItem;

fn main() {
    let r: HiearchyItem<_> = algo::to_binary_tree(vec![1, 2, 1, 10, 5, 2, 1, 2, 1]).into();

    println!("{}", r.print_tree());
}
