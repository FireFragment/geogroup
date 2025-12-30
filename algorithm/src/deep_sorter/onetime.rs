use std::{collections::HashMap, fmt::Debug};

use super::*;

/// Same as [`sort_to_binary_tree`], but assumes that the points are ordered.
///
/// Panics on `input.is_empty()`
pub fn sort_ordered_to_binary_tree<Item: SortableItem>(mut items: impl Iterator<Item=Item>) -> BinTree<Item, (), ()> {
    #[derive(Clone, Debug)]
    struct CameraPathComponent {
        pub child_index: HorizontalIdx,
        pub inner_separation: Distance,
    }
    #[derive(Clone, Debug)]
    struct CameraInfo<Item: SortableItem> {
        pub last_point: Item::Position,
        pub path_to_last_point: Vec<CameraPathComponent>
    }

    let first_point = items.next().expect("to_binary_tree called with empty vector");
    let mut the_bintree = BinTree::Leaf(first_point, ());
    let BinTree::Leaf(ref first_point, ()) = the_bintree else {panic!()};
    let mut cameras_paths: HashMap<CameraId, CameraInfo<Item>> = HashMap::new();
    cameras_paths.insert(first_point.get_camera(), CameraInfo {
        last_point: first_point.get_position().expect("POSERR"),
        path_to_last_point: Vec::new()
    });

    // We build the tree form left to right
    for item_to_append in items {
        let item_to_append_cam = item_to_append.get_camera();
        let item_to_append_pos = item_to_append.get_position().expect("POSERR");

        let camera_info = &cameras_paths[&item_to_append_cam];
        let distance_from_prev_point =
            camera_info.last_point.distance(&item_to_append_pos);


        let (
            node_to_replace,
            new_item_parent_path // Path to the parent of the newly added item
        ) = {
            let mut current_node = &mut the_bintree;
            let mut current_node_path = Vec::new();
            // Go through the tree according to `camera_info.path_to_last_point`
            // until we find a good place to place our new point, ie. the group closest
            // to the root such that its `inner_separation` is lower than the distance between
            // the new item and the previous item (`distance_from_prev_point`). Once we found
            // it, we write it to `current_node`.
            for path_component in &camera_info.path_to_last_point {
                if path_component.inner_separation < distance_from_prev_point {
                    // We found a good place to place our point
                    break;
                } else {
                    // Descend deeper into the tree
                    let BinTree::InnerNode(group) = current_node else {
                        // TODO: Either confirm 100% this can't happen or handle it more gracefully
                        panic!("Path didn't work out - unexpected leaf")
                    };

                    current_node = &mut group[path_component.child_index.clone()];
                    current_node_path.push(path_component.clone());
                }
            };

            (current_node, current_node_path)
        };

        // Update camera paths which point to the moved subtree
        for camera_info in cameras_paths.values_mut() {
            let is_in_moved_subtree = camera_info.path_to_last_point
                .iter()
                .map(|info| info.child_index.clone())
                .zip(new_item_parent_path.iter().map(|info| info.child_index.clone()))
                .all(|(a, b)| a == b);
            if is_in_moved_subtree
            {
                camera_info.path_to_last_point.insert(new_item_parent_path.len(), CameraPathComponent {
                    child_index: HorizontalIdx::Left,
                    inner_separation: node_to_replace
                        .get_leftmost_leaf().0.get_position()
                        .expect("POSERR").distance(&item_to_append_pos)
                });
            }
            //.starts_with(&new_item_parent_path) {}
        }

        let mut new_item_path = new_item_parent_path;
        new_item_path.push(CameraPathComponent { child_index: HorizontalIdx::Right, inner_separation: distance_from_prev_point });

        // TODO: Make sure that all camera paths are updated according to the move of `node_to_replace`
        take_mut::take(node_to_replace, |prev_val| BinTree::InnerNode(BTInnerNode {
            children: Box::new([prev_val, BinTree::Leaf(item_to_append, ())]),
            inner_node_data: (),
            node_data: ()
        }));

        cameras_paths.insert(item_to_append_cam, CameraInfo {
            last_point: item_to_append_pos,
            path_to_last_point: new_item_path
        });

        debug_assert!(
            cameras_paths.values().all(|info| info.path_to_last_point.is_sorted_by_key(|comp| Distance::MAX - comp.inner_separation)),
            "Sorting behaves badly: some of camera paths are not sorted by separation: {:?}",
            cameras_paths.values().map(|info| &info.path_to_last_point).collect_vec()
        );
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

    //points.sort_by_key(|p| -> Item::Time {p.get_time()});
    sort_ordered_to_binary_tree(points.into_iter()
        .filter(|point| point.get_position()
            .inspect_err(|e| log::trace!("Silently dropping a file - reason:\n{e:?}")).is_ok()
        )                                             // TODO: Don't just ignore items without position.
                                                      // Once fixed, edit all portions of the code marked with POSERR
        .sorted_by_key(|p| p.get_time())
    )
}
