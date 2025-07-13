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
        pub trait ExperimentTr<'a>: Sized {
            type InnerType;

            fn map<New: 'a, F: Fn(Self::InnerType) -> New>(
                &'a self,
                fun: F,
            ) -> impl ExperimentTr<'a, InnerType = New> {
                Mapper { orig: self, fun }
            }
        }

        pub struct Base<'a>(&'a u8);

        impl<'a> ExperimentTr<'a> for Base<'a> {
            type InnerType = u8;
        }

        pub struct Mapper<'a, Orig: ExperimentTr<'a>, New: 'a, F: Fn(Orig::InnerType) -> New> {
            pub orig: &'a Orig,
            pub fun: F,
        }

        impl<'a, Orig: ExperimentTr<'a>, New: 'a, F: Fn(Orig::InnerType) -> New> ExperimentTr<'a>
            for Mapper<'a, Orig, New, F>
        {
            type InnerType = New;
        }

        let reference = &8;

        let binding = Base(reference);
        let mapped_1 = binding.map(|f| f); //.unwrap();
        let mapped_2 = mapped_1.map(|f| f); //.unwrap();
    }

    #[test]
    fn test_lazy_hierarchy() {}
}
