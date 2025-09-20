use std::collections::HashMap;

use super::*;

/// Same as [`sort_to_binary_tree`], but assumes that the points are ordered.
///
/// Panics on `input.is_empty()`
pub fn sort_ordered_to_binary_tree<Item: SortableItem>(mut items: impl Iterator<Item=Item>) -> BinTree<Item, (), ()> {
    // We build the tree form left to right
    let first_point = items.next().expect("to_binary_tree called with empty vector");
    let mut the_bintree = BinTree::Leaf(first_point, ());
    let BinTree::Leaf(ref first_point, ()) = the_bintree else {panic!()};
    struct CameraPathComponent {
        pub child_index: HorizontalIdx,
        pub inner_separation: Distance,
    }
    struct CameraInfo<Item: SortableItem> {
        pub last_point: Item::Position,
        pub path_to_last_point: Vec<CameraPathComponent>
    }
    let mut cameras_paths: HashMap<CameraId, CameraInfo<Item>> = HashMap::new();
    cameras_paths.insert(first_point.get_camera(), CameraInfo {
        last_point: first_point.get_position().expect("POSERR"),
        path_to_last_point: Vec::new()
    });

    for item_to_append in items {
        let camera_info = &cameras_paths[&item_to_append.get_camera()];
        let distance_from_prev_point =
            camera_info.last_point.distance(&item_to_append.get_position().expect("POSERR"));


        let node_to_replace = {
            let mut current_node = &mut the_bintree;
            // Go through the tree according to `camera_info.path_to_last_point`
            // until we find a good place to place our new point, ie. the group closest
            // to the root such that its `inner_separation` is lower than the distance between
            // the new item and the previous item (`distance_from_prev_point`). Once we found
            // it, we write it to `current_node`
            for path_component in &camera_info.path_to_last_point {
                // We found the spot for our new point
                if path_component.inner_separation < distance_from_prev_point {
                    // We found a good place to place our point
                    break;
                } else {
                    let BinTree::InnerNode(group) = current_node else {
                        // TODO: Either confirm 100% this can't happen or handle it more gracefully
                        panic!("Path didn't work out - unexpected leaf")
                    };

                    current_node = &mut group[path_component.child_index.clone()];
                }
            };

            current_node
        };


        take_mut::take(node_to_replace, |prev_val| BinTree::InnerNode(BTInnerNode {
            children: Box::new([BinTree::Leaf(item_to_append, ()), prev_val]),
            inner_node_data: (),
            node_data: ()
        }));
    }


    the_bintree
}



/*struct PreprocessedPoint<P, Fail> {
    point: P,
    distance_to_next: u64,
    /// List of failed points between this and the next point
    failed_until_next: Vec<Fail>
}*/


/// Sort points to binary tree.
///
/// The algorithm always cuts points into two groups by the biggest distance
/// and then it recurses again on theese two groups
///
/// Panics on `input.is_empty()`
pub fn sort_to_binary_tree<Item: SortableItem>(points: impl IntoIterator<Item=Item>) -> BinTree<Item, (), ()> {
    // TODO: Don't just ignore items without position.
    // Once fixed, edit all portions of the code marked with POSERR
    let mut points: Vec<_> = points.into_iter().filter(|point| point.get_position().is_ok()).collect();

    points.sort_by_key(|p| -> Item::Time {p.get_time()});
    sort_ordered_to_binary_tree(points.into_iter())
}
