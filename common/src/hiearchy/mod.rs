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

    mod err_repro {

        /// Simple reproduction of the lifetime error
        fn err_repro() {
            pub trait AsGroupRef: Sized {
                type GroupRef<'a>: GroupRef
                where
                    Self: 'a;

                fn as_group_ref(&self) -> Self::GroupRef<'_>;
            }

            pub trait GroupRef {}

            fn map<'a, T: GroupRef>(
                _: T,
            ) -> impl AsGroupRef<GroupRef<'a> = impl GroupRef + use<'a, T>> {
                AsGrMapper
            }

            pub struct Base;
            impl GroupRef for Base {}

            pub struct AsGrMapper;
            pub struct GrMapper<'a>(&'a AsGrMapper);

            impl<'a> GroupRef for GrMapper<'a> {}

            impl AsGroupRef for AsGrMapper {
                type GroupRef<'a> = GrMapper<'a>;

                fn as_group_ref(&self) -> Self::GroupRef<'_> {
                    todo!()
                }
            }

            let binding = Base;
            let mapped_1 = map(binding);
            let mapped_2 = map(mapped_1.as_group_ref());
        }
    }

    #[test]
    fn test_lazy_hierarchy() {}
}
