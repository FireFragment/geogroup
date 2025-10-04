use super::*;

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
fn test_map() {
    use concrete::*;
    let hierarchy = ConcreteHiearchy::new(Group::new(
        vec![
            Node::new_leaf(Leaf::new(10, String::from("a leaf"))),
            Node::new_group(Group::new(Vec::new(), true, String::from("a subgroup"))),
        ],
        false,
        String::from("root group"),
    ));

    let ext_var: Vec<i32> = Vec::new(); // Test that the code compiles even if the closure references an external variable
    let binding = hierarchy
        .root()
        .map_group_data(|group_ref| !group_ref.group_data() && ext_var.len() == 0);

    let collected = binding.collect_to_concrete().unwrap();

    dbg!(&collected);

    let str_leaf = String::from("a leaf");
    let str_subgroup = String::from("a subgroup");
    let str_root_group = String::from("root group");
    let target = Group::new(
        vec![
            Node::new_leaf(Leaf::new(&10, &str_leaf)),
            Node::new_group(Group::new(Vec::new(), false, &str_subgroup)),
        ],
        true,
        &str_root_group,
    );

    assert_eq!(collected, target);
}

#[test]
fn test_map_leaf_data() {
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

    let ext_coeficient = 2; // Test that the code compiles even if the closure references an external variable
    let binding = hierarchy
        .root()
        .map_leaf_data(|leaf_ref| leaf_ref.leaf_data() * ext_coeficient);
    let collected = binding.collect_to_concrete().unwrap();

    dbg!(&collected);

    let str_leaf = String::from("a leaf");
    let str_nested_leaf = String::from("nested leaf");
    let str_subgroup = String::from("a subgroup");
    let str_root_group = String::from("root group");
    let target = Group::new(
        vec![
            Node::new_leaf(Leaf::new(20, &str_leaf)), // 10 * 2 = 20
            Node::new_group(Group::new(
                vec![Node::new_leaf(Leaf::new(40, &str_nested_leaf))], // 20 * 2 = 40
                &true,
                &str_subgroup,
            )),
        ],
        &false,
        &str_root_group,
    );

    assert_eq!(collected, target);
}

#[test]
fn test_with_idx() {
    use concrete::*;

    
    let hierarchy = ConcreteHiearchy::new(Group::new(
        vec![
            Node::new_leaf(Leaf::new(10, String::from("leaf1"))),
            Node::new_leaf(Leaf::new(20, String::from("leaf2"))),
            Node::new_group(Group::new(
                vec![
                    Node::new_leaf(Leaf::new(30, String::from("nested_leaf1"))),
                    Node::new_leaf(Leaf::new(40, String::from("nested_leaf2"))),
                ],
                true,
                String::from("subgroup"),
            )),
        ],
        false,
        String::from("root"),
    ));

    let indexed = hierarchy.root().with_idx();
    
    // Test root node has index 0
    let root_node_data = indexed.node_data();
    assert_eq!(root_node_data.data, &String::from("root"));
    assert_eq!(root_node_data.index, 0);
    
    // Test children have correct indices
    let children: Vec<_> = indexed.get_children().unwrap().collect();
    assert_eq!(children.len(), 3);
    
    // First leaf should have index 0
    let leaf1_node_data = children[0].node_data();
    assert_eq!(leaf1_node_data.data, &String::from("leaf1"));
    assert_eq!(leaf1_node_data.index, 0);
    
    // Second leaf should have index 1
    let leaf2_node_data = children[1].node_data();
    assert_eq!(leaf2_node_data.data, &String::from("leaf2"));
    assert_eq!(leaf2_node_data.index, 1);
    
    // Subgroup should have index 2
    let subgroup_node_data = children[2].node_data();
    assert_eq!(subgroup_node_data.data, &String::from("subgroup"));
    assert_eq!(subgroup_node_data.index, 2);
    
    // Test nested children have correct indices
    if let crate::NodeRef::Group(subgroup) = &children[2] {
        let nested_children: Vec<_> = subgroup.get_children().unwrap().collect();
        assert_eq!(nested_children.len(), 2);
        
        let nested1_node_data = nested_children[0].node_data();
        assert_eq!(nested1_node_data.data, &String::from("nested_leaf1"));
        assert_eq!(nested1_node_data.index, 0);
        
        let nested2_node_data = nested_children[1].node_data();
        assert_eq!(nested2_node_data.data, &String::from("nested_leaf2"));
        assert_eq!(nested2_node_data.index, 1);
    } else {
        panic!("Expected subgroup to be a Group");
    }
}
