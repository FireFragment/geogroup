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
        use std::marker::PhantomData;

        /// Simple reproduction of the lifetime error
        fn err_repro() {
            pub trait AsGroupRef {
                type AssocType;
                type GroupRef<'a>: GroupRef<AssocType = Self::AssocType>
                where
                    Self: 'a;

                fn as_group_ref(&self) -> Self::GroupRef<'_>;
            }

            pub trait GroupRef {
                type AssocType;
            }

            pub struct Base;
            impl GroupRef for Base {
                type AssocType = u8;
            }

            pub struct AsGrMapper<Gr: GroupRef, T>(Gr, T);
            pub struct GrMapper<'a, Gr: GroupRef, T>(Gr, &'a AsGrMapper<Gr, T>);

            impl<'a, Gr: GroupRef, T> GroupRef for GrMapper<'a, Gr, T> {
                type AssocType = T;
            }

            impl<Gr: GroupRef, T> AsGroupRef for AsGrMapper<Gr, T> {
                type AssocType = T;
                type GroupRef<'a>
                    = GrMapper<'a, Gr, T>
                where
                    Self: 'a;

                fn as_group_ref(&self) -> Self::GroupRef<'_> {
                    todo!()
                }
            }

            fn map<'a, Gr: GroupRef + 'a, T: 'a>(gr: Gr, t: T) -> impl AsGroupRef<AssocType = T> {
                AsGrMapper(gr, t)
            }

            let binding = Base;
            let mapped_1 = map(binding, 8);
            let gr = map(mapped_1.as_group_ref(), 8);
        }
    }

    #[test]
    fn test_lazy_hierarchy() {}
}
