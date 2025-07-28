pub mod concrete;
pub mod lazy;
pub use concrete::ConcreteHiearchy as Concrete;
pub use lazy::AsGroupRef as Lazy;
#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

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

    /// Simple reproduction of the lifetime error
    fn err_repro() {
        pub trait ExperimentTrait<'a>: Sized {}

        fn my_fun<'a, T: ExperimentTrait<'a>>(_: &'a T) -> impl ExperimentTrait<'a> {
            ExperimentInstance
        }

        pub struct Base<'a>(&'a u8);

        impl<'a> ExperimentTrait<'a> for Base<'a> {}

        pub struct ExperimentInstance;

        impl<'a> ExperimentTrait<'a> for ExperimentInstance {}

        let binding = ExperimentInstance;
        let mapped_1 = my_fun(&binding);
        let mapped_2 = my_fun(&mapped_1);
    }

    #[test]
    fn test_lazy_hierarchy() {}
}
