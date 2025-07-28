pub mod concrete;
pub mod lazy;
pub use concrete::ConcreteHiearchy as Concrete;
pub use lazy::AsGroupRef as Lazy;
#[cfg(test)]
mod tests {
    use lazy::{AsGroupRefUtils, GroupRef, GroupRefUtils, LeafRef};

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
            .map_group_data(|group_ref| !group_ref.group_metadata() && ext_var.len() == 0);

        binding
            .root()
            .map_group_data(|group_ref| !group_ref.group_metadata() && ext_var.len() == 0);

        /*let collected = binding.collect_to_concrete().unwrap();

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

        assert_eq!(collected, target);*/
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
        let binding = hierarchy.map_leaf_data(|leaf_ref| leaf_ref.leaf_metadata() * ext_coeficient);
        let collected = binding.collect_to_concrete().unwrap();

        dbg!(&collected);

        let str_leaf = String::from("a leaf");
        let str_nested_leaf = String::from("nested leaf");
        let str_subgroup = String::from("a subgroup");
        let str_root_group = String::from("root group");
        let target = ConcreteHiearchy::new(Group::new(
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
        ));

        assert_eq!(collected, target);
    }
}
