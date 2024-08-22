use crate::*;

/// Sort points by the geogroup algorithm
///
/// The generic argument `P: Point + Clone` should be fast to clone.
pub fn sort<P: Point + Clone>(points: Vec<P>, depth: Depth) -> HiearchyItem<P> {
    flatten(sort_to_binary_tree(points), depth)
}

/// Sort points to binary tree.
///
/// The algorithm always cuts points into two groups by the biggest distance
/// and then it recurses again on theese two groups
///
/// Panics on `input.is_empty()`
pub fn sort_to_binary_tree<P: Point>(points: Vec<P>) -> BinTree<P, ()> {
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

    sort_to_binary_tree_with_distances(points_with_distances, last_point)
}

/// The recursive part of [`to_binary_tree`]
///
/// This function takes vector of points alongside their distances to the next one. \
/// However, the last point has no "next" point, so it can't have any distance to it,
/// so it's provided as a separate argument
fn sort_to_binary_tree_with_distances<P>(
    input: Vec<(P, Distance)>,
    last_point: P,
) -> BinTree<P, ()> {
    if input.len() == 0 {
        return BinTree::Leaf(last_point);
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

    BinTree::InnerNode {
        children: Box::new([
            sort_to_binary_tree_with_distances(first_group, last_from_first_group.0),
            sort_to_binary_tree_with_distances(second_group, last_point),
        ]),
        data: (),
    }
}

/// "Flatten" a binary tree of points so that only bigger spaces between points are separated to different groups
///
/// The generic argument `P: Point + Clone` should be fast to clone.
pub fn flatten<P: Point + Clone>(bintree: BinTree<P, ()>, depth: Depth) -> HiearchyItem<P> {
    flatten_inner(bintree, depth).flattened_group
}

/// Flatten, but also return addidional data in [`FlattenRet`] useful for recursion
fn flatten_inner<P: Point + Clone>(bintree: BinTree<P, ()>, depth: Depth) -> FlattenRet<P> {
    match bintree {
        BinTree::InnerNode { children, data: _ } => {
            let [first, second] = children.map(|subtree| flatten_inner(subtree, depth));
            let highest_inner_distance = first.last_point.distance(&second.first_point);
            let first_point = first.first_point.clone();
            let last_point = second.last_point.clone();

            let mut final_group = Vec::new();

            dissolve_if_needed(first, &mut final_group, depth, highest_inner_distance);
            dissolve_if_needed(second, &mut final_group, depth, highest_inner_distance);

            FlattenRet {
                highest_inner_distance,
                first_point,
                last_point,
                flattened_group: HiearchyItem::Group(final_group),
            }
        }
        BinTree::Leaf(point) => FlattenRet {
            highest_inner_distance: 0,
            first_point: point.clone(),
            last_point: point.clone(),
            flattened_group: HiearchyItem::Item(point),
        },
    }
}

fn dissolve_if_needed<P: Point>(
    group_to_dissolve: FlattenRet<P>,
    final_group: &mut Vec<HiearchyItem<P>>,
    depth: Depth,
    highest_inner_distance: Distance,
) {
    // How "weak" is group
    let first_group_weakness = (Depth::MAX as u128
        * group_to_dissolve.highest_inner_distance as u128
        / highest_inner_distance as u128) as Depth;

    // lower or equal condition to not trigger the panic in case that highest_inner_distance = 0
    if first_group_weakness <= depth {
        final_group.push(group_to_dissolve.flattened_group);
    } else {
        let HiearchyItem::Group(mut group_to_dissolve_g) = group_to_dissolve.flattened_group else {
            panic!(
                "
                    Probably faulty `FlattenRet` with highest_inner_distance != 0, but it's {}.
                    (if it really is 0, that would be dumb...)
                ",
                group_to_dissolve.highest_inner_distance
            )
        };

        final_group.append(&mut group_to_dissolve_g);
    }
}

struct FlattenRet<P: Point> {
    /// 0, if it's not a group
    highest_inner_distance: Distance,
    last_point: P,
    first_point: P,
    flattened_group: HiearchyItem<P>,
}
