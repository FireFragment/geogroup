pub mod concrete;
pub mod lazy;
pub use concrete::ConcreteHiearchy as Concrete;
pub(crate) use either::Either;
pub use lazy::LazyHiearchy as Lazy;
#[cfg(test)]
mod tests {
    use lazy::{GroupRef, LazyHiearchyUtils, NoLoadingLazyHiearchyUtils};

    use super::*;

    #[test]
    fn map_nodes_map_groups() {
        use concrete::*;
        let hiearchy = ConcreteHiearchy::new(Group::new(
            vec![
                Node::new_leaf(Leaf::new(10, String::from("a leaf"))),
                Node::new_group(Group::new(Vec::new(), true, String::from("a subgroup"))),
            ],
            false,
            String::from("root group"),
        ));

        let h = hiearchy
            .map_nodes(|g| format!("mapped {g}"))
            .map_groups(|g| !g);
        let collected = h.collect_to_concrete().unwrap();

        dbg!(&collected);

        let target = ConcreteHiearchy::new(Group::new(
            vec![
                Node::new_leaf(Leaf::new(&10, String::from("mapped a leaf"))),
                Node::new_group(Group::new(
                    Vec::new(),
                    false,
                    String::from("mapped a subgroup"),
                )),
            ],
            true,
            String::from("mapped root group"),
        ));

        assert_eq!(collected, target);

        //let aa: Vec<_> = mh.root().get_children().to_result().unwrap().collect();

        //mh.map_groups(|n: &bool| !n);
    }

    #[test]
    fn test_lazy_hierarchy() {}
}
