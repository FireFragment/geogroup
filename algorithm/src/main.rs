use geogroup_algo::*;
use geogroup_common::hiearchy::lazy::NoLoadingLazyHiearchyUtils;

fn main() {
    println!(
        "{:#?}",
        deep_sorter::sort_ordered_to_binary_tree::<u64>(vec![1, 2, 1, 10, 5, 2, 1, 2, 1])
            .collect_to_concrete()
            .unwrap()
    );
}
