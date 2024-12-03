use geogroup_algo::*;

fn main() {
    println!(
        "{}",
        sort(
            [1, 2, 1, 10, 5, 2, 1, 2, 1]
                .into_iter()
                .map(|n| (n, ()))
                .collect(),
            &Params::default()
        )
        .print_tree_points_only()
    );
}
