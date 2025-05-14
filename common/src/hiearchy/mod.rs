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

    hello(mapped);
}

fn hello<
    'a,
    T: Lazy<StructureErr = Infallible, DoesLoading = Infallible> + std::marker::Sized + 'a,
>(
    t: T,
) where
    T::GroupMetadata<'a>: std::fmt::Debug,
{
    use lazy::NoLoadingLazyHiearchyUtils;
    // Probable cause of the error: caller can choose 'a to be ANY lifetime,
    // including lifetimes longer than the function `hello` itself.
    //
    // This means, that the `T::GroupMetadata<'a>: std::fmt::Debug` bound applies
    // ONLY to some caller chosen lifetime 'a. If this lifetime is longer than
    // the function body, then `fmt::Debug` is NOT implemented for `t`, because
    // in that case, the `T::GroupMetadata<'a>: std::fmt::Debug` bound applies
    // only when `t` lives longer than the function itself, which it obviously
    // can't.
    let c = t.collect_to_concrete().unwrap();

    debug_it_two(c);
}

fn debug_it_two<G: std::fmt::Debug, L, N>(_x: Concrete<G, L, N>) {
    //dbg!(h);
}
