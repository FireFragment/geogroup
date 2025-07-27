use super::*;
use lazy::AsGroupRefUtils;

#[test]
fn test_concrete_hierarchy() {
    use concrete::*;
    let hierarchy = ConcreteHiearchy::new(Group::new(
        vec![
            Node::new_leaf(Leaf::new(10, String::from("a leaf"))),
            Node::new_group(Group::new(Vec::new(), true, String::from("a subgroup"))),
        ],
        false,
        String::from("root group"),
    ));

    let collected = hierarchy.collect_to_concrete().unwrap();

    dbg!(&collected);

    let str_leaf = String::from("a leaf");
    let str_subgroup = String::from("a subgroup");
    let str_root_group = String::from("root group");
    let target = ConcreteHiearchy::new(Group::new(
        vec![
            Node::new_leaf(Leaf::new(&10, &str_leaf)),
            Node::new_group(Group::new(Vec::new(), &true, &str_subgroup)),
        ],
        &false,
        &str_root_group,
    ));

    assert_eq!(collected, target);
}

#[test]
fn test_map_groups() {
    use concrete::*;
    let hierarchy = ConcreteHiearchy::new(Group::new(
        vec![
            Node::new_leaf(Leaf::new(10, String::from("a leaf"))),
            Node::new_group(Group::new(Vec::new(), true, String::from("a subgroup"))),
        ],
        false,
        String::from("root group"),
    ));

    let binding = hierarchy.map_groups(|g, _| !g);
    let collected = binding.collect_to_concrete().unwrap();

    dbg!(&collected);

    let str_leaf = String::from("a leaf");
    let str_subgroup = String::from("a subgroup");
    let str_root_group = String::from("root group");
    let target = ConcreteHiearchy::new(Group::new(
        vec![
            Node::new_leaf(Leaf::new(&10, &str_leaf)),
            Node::new_group(Group::new(Vec::new(), false, &str_subgroup)),
        ],
        true,
        &str_root_group,
    ));

    assert_eq!(collected, target);
}

#[test]
fn test_map_leaves() {
    use concrete::*;
    let hierarchy = ConcreteHiearchy::new(Group::new(
        vec![
            Node::new_leaf(Leaf::new(10, String::from("a leaf"))),
            Node::new_group(Group::new(
                vec![Node::new_leaf(Leaf::new(20, String::from("nested leaf")))],
                true,
                String::from("a subgroup"),
            )),
        ],
        false,
        String::from("root group"),
    ));

    let binding = hierarchy.map_leaves(|leaf_data, _| leaf_data * 2);
    let collected = binding.collect_to_concrete().unwrap();

    dbg!(&collected);

    let str_leaf = String::from("a leaf");
    let str_nested_leaf = String::from("nested leaf");
    let str_subgroup = String::from("a subgroup");
    let str_root_group = String::from("root group");
    let target = ConcreteHiearchy::new(Group::new(
        vec![
            Node::new_leaf(Leaf::new(20, &str_leaf)),
            Node::new_group(Group::new(
                vec![Node::new_leaf(Leaf::new(40, &str_nested_leaf))],
                &true,
                &str_subgroup,
            )),
        ],
        &false,
        &str_root_group,
    ));

    assert_eq!(collected, target);
}
