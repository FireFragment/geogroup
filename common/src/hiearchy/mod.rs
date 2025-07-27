pub mod concrete;
pub mod lazy;
pub use concrete::ConcreteHiearchy as Concrete;
pub use lazy::AsGroupRef as Lazy;
#[cfg(test)]
mod tests {
    use lazy::AsGroupRefUtils;

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

        let binding = hierarchy.map_group_data(|g| !g);
        let collected = binding.collect_to_concrete().unwrap();

        //drop(binding);

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
    fn test_lazy_hierarchy() {}
}
