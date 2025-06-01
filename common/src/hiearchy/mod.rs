pub mod concrete;
pub mod lazy;
use std::convert::Infallible;

pub use concrete::ConcreteHiearchy as Concrete;
use concrete::Group;
pub use lazy::LazyHiearchy as Lazy;
#[cfg(test)]
mod tests {
    use lazy::{GroupRef, LazyHiearchyUtils, NoLoadingLazyHiearchyUtils};

    use super::*;

    #[test]
    /*fn test_concrete_hierarchy() {
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
        drop(aa);

        drop(mh);
        //mh.map_groups(|n: &bool| !n);
    }*/
    #[test]
    fn test_lazy_hierarchy() {}
}

fn hello_outer() {
    use lazy::LazyHiearchyUtils;
    let original = Concrete::<(), (), ()>::new(Group::new(Vec::new(), (), ()));
    let mapped = original.map_nodes(|()| ());

    // This doesn't work, here's my theory:
    // map_nodes specifies in its signature that it returns `impl LazyHiearchy<GroupMetadata<'b> = Self::GroupMetadata<'b>>`
    // for an arbitrary lifetime 'b chosen WHEN CALLING THE FUNCTION.
    // However, this is different from `impl for<'a> LazyHiearchy<GroupMetadata<'a> = Self::GroupMetadata<'a>>`
    // because that would be for _arbitrary_ lifetime 'a that could be chosen anytime GroupMetadata is used.
    //
    // Moreover, 'b is limited by map_nodes in some ways, such as requiring that 'b outlives NewNodeMetadata,
    // so it can't be used here.
    hello(mapped);
}

fn hello<T: Lazy<StructureErr = Infallible, DoesLoading = Infallible> + std::marker::Sized>(t: T)
where
    // Very relevant: https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats
    for<'a> T::GroupMetadata<'a>: std::fmt::Debug,
{
    use lazy::NoLoadingLazyHiearchyUtils;
    let c = t.collect_to_concrete().unwrap();

    debug_it_two(c);
}

fn debug_it_two<G: std::fmt::Debug, L, N>(_x: Concrete<G, L, N>) {
    //dbg!(h);
}
