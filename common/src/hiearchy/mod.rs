pub mod concrete;
pub mod lazy;
pub use concrete::ConcreteHiearchy as Concrete;
pub use lazy::LazyHiearchy as Lazy;
#[cfg(test)]
mod tests {
    use lazy::{GroupRef, LazyHiearchyUtils, NoLoadingLazyHiearchyUtils};

    use super::*;

    #[test]
    fn test_concrete_hierarchy() {
        use concrete::*;
        let hiearchy = ConcreteHiearchy::new(Group::new(
            vec![
                Node::new_leaf(Leaf::new(10, String::from("a leaf"))),
                Node::new_group(Group::new(Vec::new(), true, String::from("a subgroup"))),
            ],
            false,
            String::from("root group"),
        ));

        let mh = hiearchy.map_nodes(|g| format!("mapped {g}"));

        //let aa: Vec<_> = mh.root().get_children().to_result().unwrap().collect();
        let aa = mh.collect_to_concrete();

        //mh.map_groups(|n: &bool| !n);
    }

    #[test]
    fn test_lazy_hierarchy() {}
}
