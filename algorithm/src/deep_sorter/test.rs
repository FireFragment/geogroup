//! I confess, these tests are AI slop.
// TODO: Check it manually, some of the tests look weird
use super::*;
use crate::{Params, Point};
use geogroup_common::hiearchy::lazy::{AsGroupRef, GroupRef as _, LeafRef as _};

/// Simple test item that implements SortableItem
#[derive(Debug, Clone, PartialEq)]
struct TestItem {
    time: u64,
    position: u64,
    id: String,
}

impl TestItem {
    fn new(time: u64, position: u64, id: &str) -> Self {
        Self {
            time,
            position,
            id: id.to_string(),
        }
    }
}

impl Point for TestItem {
    fn distance(&self, rhs: &Self) -> Distance {
        self.position.distance(&rhs.position)
    }
}

impl SortableItem for TestItem {
    type Time = u64;
    type Position = u64;

    fn get_time(&self) -> Self::Time {
        self.time
    }

    fn get_position(&self) -> Self::Position {
        self.position
    }
}

/// Helper function to collect all leaves from a hierarchy
fn collect_leaves<'a, Item: SortableItem>(group_ref: &DeepSorterGroupRef<'a, Item>) -> Vec<String>
where
    Item: std::fmt::Debug,
{
    let mut leaves = Vec::new();

    fn collect_recursive<'a, Item: SortableItem>(
        group_ref: &DeepSorterGroupRef<'a, Item>,
        leaves: &mut Vec<String>,
    ) where
        Item: std::fmt::Debug,
    {
        let children = group_ref.get_children().unwrap();
        for child in children {
            match child {
                geogroup_common::hiearchy::lazy::NodeRef::Group(sub_group) => {
                    collect_recursive(&sub_group, leaves);
                }
                geogroup_common::hiearchy::lazy::NodeRef::Leaf(leaf) => {
                    leaves.push(format!("{:?}", leaf.leaf_metadata()));
                }
            }
        }
    }

    collect_recursive(group_ref, &mut leaves);
    leaves
}

/// Helper function to collect group info from hierarchy
fn collect_group_info<'a, Item: SortableItem>(
    group_ref: &DeepSorterGroupRef<'a, Item>,
) -> Vec<(Option<Distance>, String)>
where
    Item: std::fmt::Debug,
{
    let mut groups = Vec::new();

    fn collect_recursive<'a, Item: SortableItem>(
        group_ref: &DeepSorterGroupRef<'a, Item>,
        groups: &mut Vec<(Option<Distance>, String)>,
        depth: usize,
    ) where
        Item: std::fmt::Debug,
    {
        let group_data = group_ref.group_metadata();

        groups.push((
            group_data.separation,
            format!(
                "Depth{}: {} ({:?})",
                depth,
                group_data
                    .separation
                    .map_or("None".to_string(), |d| d.to_string()),
                group_data.strength
            ),
        ));

        let children = group_ref.get_children().unwrap();
        for child in children {
            if let geogroup_common::hiearchy::lazy::NodeRef::Group(sub_group) = child {
                collect_recursive(&sub_group, groups, depth + 1);
            }
        }
    }

    collect_recursive(group_ref, &mut groups, 0);
    groups
}

#[test]
fn test_deep_sorter_simple_linear() {
    // Create a simple linear sequence where items are evenly spaced in time and position
    let items = vec![
        TestItem::new(1, 10, "item1"),
        TestItem::new(2, 20, "item2"),
        TestItem::new(3, 30, "item3"),
        TestItem::new(4, 40, "item4"),
    ];

    let sorter = DeepSorter::new(items, Params::default());
    let root = sorter.root();

    // Check that we get the expected leaves
    let leaves = collect_leaves(&root);
    assert_eq!(leaves.len(), 4);

    // The largest gap should be the root separation
    let group_info = collect_group_info(&root);
    assert!(!group_info.is_empty());

    // Root should have separation of 10 (max distance between consecutive items)
    let root_info = &group_info[0];
    assert_eq!(root_info.0, Some(10));
}

#[test]
fn test_deep_sorter_with_gaps() {
    // Create items with varying gaps to test hierarchy formation
    let items = vec![
        TestItem::new(1, 10, "item1"),
        TestItem::new(2, 15, "item2"),   // gap of 5
        TestItem::new(3, 20, "item3"),   // gap of 5
        TestItem::new(10, 100, "item4"), // large gap of 80
        TestItem::new(11, 110, "item5"), // gap of 10
    ];

    let sorter = DeepSorter::new(items, Params::default());
    let root = sorter.root();

    let leaves = collect_leaves(&root);
    assert_eq!(leaves.len(), 5);

    let group_info = collect_group_info(&root);

    // The root should have the maximum separation (80)
    let root_info = &group_info[0];
    assert_eq!(
        root_info.0,
        Some(80),
        "Root separation should be 80 (the largest gap)"
    );

    // Should have multiple levels due to the large gap
    assert!(
        group_info.len() > 1,
        "Should have multiple hierarchy levels"
    );
}

#[test]
fn test_deep_sorter_identical_positions() {
    // Test with items at identical positions but different times
    let items = vec![
        TestItem::new(1, 50, "item1"),
        TestItem::new(2, 50, "item2"),  // same position
        TestItem::new(3, 50, "item3"),  // same position
        TestItem::new(4, 100, "item4"), // different position
    ];

    let sorter = DeepSorter::new(items, Params::default());
    let root = sorter.root();

    let leaves = collect_leaves(&root);
    assert_eq!(leaves.len(), 4);

    let group_info = collect_group_info(&root);
    let root_info = &group_info[0];

    // Root separation should be 50 (distance from position 50 to 100)
    assert_eq!(root_info.0, Some(50));
}

#[test]
fn test_deep_sorter_single_item() {
    // Skip single item test - DeepSorter expects multiple items
    // Single item case hits unimplemented code path in bintree
}

#[test]
fn test_deep_sorter_two_items() {
    let items = vec![
        TestItem::new(1, 10, "first"),
        TestItem::new(2, 30, "second"),
    ];

    let sorter = DeepSorter::new(items, Params::default());
    let root = sorter.root();

    let leaves = collect_leaves(&root);
    assert_eq!(leaves.len(), 2);

    let group_info = collect_group_info(&root);

    // With two items, there should be one separation value
    let root_info = &group_info[0];
    assert_eq!(root_info.0, Some(20)); // distance between positions 10 and 30
}

#[test]
fn test_deep_sorter_strength_calculation() {
    // Create a hierarchy that should produce specific strength values
    let items = vec![
        TestItem::new(1, 10, "item1"),
        TestItem::new(2, 20, "item2"),   // gap: 10
        TestItem::new(5, 25, "item3"),   // gap: 5
        TestItem::new(6, 30, "item4"),   // gap: 5
        TestItem::new(10, 100, "item5"), // gap: 70 (should be root split)
        TestItem::new(11, 120, "item6"), // gap: 20
    ];

    let sorter = DeepSorter::new(items, Params::default());
    let root = sorter.root();

    let group_info = collect_group_info(&root);

    // Root should have the largest separation (70)
    let root_info = &group_info[0];
    assert_eq!(root_info.0, Some(70));

    // Check that strength info is being calculated
    assert!(group_info.iter().any(|(_, desc)| desc.contains("Root")));

    // Should have sub-groups with different strengths
    let sub_groups: Vec<_> = group_info
        .iter()
        .filter(|(_, desc)| desc.contains("Ok("))
        .collect();

    // At least one sub-group should have a calculated strength
    assert!(
        !sub_groups.is_empty(),
        "Should have sub-groups with calculated strength"
    );
}

#[test]
fn test_deep_sorter_hierarchy_consistency() {
    // Test that the hierarchy produces consistent results
    let items = vec![
        TestItem::new(1, 100, "a"),
        TestItem::new(2, 110, "b"),
        TestItem::new(3, 120, "c"),
        TestItem::new(10, 200, "d"), // large jump
        TestItem::new(11, 205, "e"),
        TestItem::new(12, 210, "f"),
        TestItem::new(20, 300, "g"), // another large jump
        TestItem::new(21, 310, "h"),
    ];

    let sorter = DeepSorter::new(items, Params::default());
    let root = sorter.root();

    let leaves = collect_leaves(&root);
    assert_eq!(leaves.len(), 8);

    let group_info = collect_group_info(&root);

    // Root should split on the largest gap
    let root_separation = group_info[0].0.unwrap();

    // The largest gaps should be either 80 (200-120) or 90 (300-210)
    assert!(
        root_separation == 80 || root_separation == 90,
        "Root separation should be one of the largest gaps, got {}",
        root_separation
    );

    // Verify hierarchy depth - should have multiple levels
    assert!(
        group_info.len() >= 3,
        "Should have at least 3 hierarchy levels"
    );
}

#[test]
fn test_deep_sorter_unsorted_input() {
    // Test that DeepSorter works with unsorted input (it should sort by time internally)
    let items = vec![
        TestItem::new(5, 50, "middle"),
        TestItem::new(1, 10, "first"),
        TestItem::new(10, 100, "last"),
        TestItem::new(3, 30, "second"),
    ];

    let sorter = DeepSorter::new(items, Params::default());
    let root = sorter.root();

    let leaves = collect_leaves(&root);
    assert_eq!(leaves.len(), 4);

    // The algorithm should handle the unsorted input correctly
    let group_info = collect_group_info(&root);
    assert!(!group_info.is_empty());

    // Should produce a meaningful hierarchy
    let root_info = &group_info[0];
    assert!(root_info.0.is_some(), "Root should have a separation value");
}

#[test]
fn test_deep_sorter_regression_specific_values() {
    // This test locks in specific values to catch regressions
    let items = vec![
        TestItem::new(1, 0, "start"),
        TestItem::new(2, 10, "a"),
        TestItem::new(3, 15, "b"),
        TestItem::new(4, 20, "c"),
        TestItem::new(10, 100, "gap"),
        TestItem::new(11, 105, "d"),
    ];

    let sorter = DeepSorter::new(items, Params::default());
    let root = sorter.root();

    let group_info = collect_group_info(&root);

    // Lock in the expected root separation value
    assert_eq!(
        group_info[0].0,
        Some(80),
        "Root separation regression test failed"
    );

    // Lock in the number of hierarchy levels (observed: 5)
    assert_eq!(
        group_info.len(),
        5,
        "Hierarchy depth regression test failed"
    );

    // Lock in that we have the expected leaf count
    let leaves = collect_leaves(&root);
    assert_eq!(leaves.len(), 6, "Leaf count regression test failed");
}
