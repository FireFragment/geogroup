use std::ops::Index;

use crate::*;

/// Sort points to binary tree.
///
/// The algorithm always cuts points into two groups by the biggest distance
/// and then it recurses again on theese two groups
///
/// Panics on `input.is_empty()`
pub fn to_binary_tree<P: Point>(points: Vec<P>) -> HiearchyItem<P> {
    assert!(
        !points.is_empty(),
        "to_binary_tree called with empty vector",
    );

    let mut points_with_distances = Vec::new();
    let mut points_iter = points.into_iter().peekable();

    let last_point = loop {
        // This unwrap suceeds:
        //  - In the first iteration, it has been asserted that there are at least 2 points.
        //  - In following iterations, it has been peeked on the next element.
        //    If there wasn't one, the loop would be broken out of.
        let current_point = points_iter.next().unwrap();
        if let Some(next_point) = points_iter.peek() {
            let distance_to_next = current_point.distance(next_point);
            points_with_distances.push((current_point, distance_to_next));
        } else {
            break current_point;
        }
    };

    to_binary_tree_with_distances(points_with_distances, last_point)
}

/// The recursive part of [`to_binary_tree`]
///
/// This function takes vector of points alongside their distances to the next one. \
/// However, the last point has no "next" point, so it can't have any distance to it,
/// so it's provided as a separate argument
fn to_binary_tree_with_distances<P>(input: Vec<(P, Distance)>, last_point: P) -> HiearchyItem<P> {
    if input.len() == 0 {
        return HiearchyItem::Item(last_point);
    };

    let idx_of_max_distance_to_next = input
        .iter()
        .enumerate()
        .max_by_key(|(_idx, (_, distance_to_next))| *distance_to_next)
        .unwrap()
        .0;

    let mut first_group = input;
    let second_group = first_group.split_off(idx_of_max_distance_to_next + 1);

    let last_from_first_group = first_group.pop().expect("split_off returned empty?");

    HiearchyItem::Group(vec![
        to_binary_tree_with_distances(first_group, last_from_first_group.0),
        to_binary_tree_with_distances(second_group, last_point),
    ])
}

fn flatten<P: Point>(bintree: HiearchyItem<P>) -> (Distance, HiearchyItem<P>) {
    todo!()
}
