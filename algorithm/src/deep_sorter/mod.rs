use std::convert::Infallible;
mod onetime;

use crate::*;
pub use onetime::*;

pub struct DeepSorter<Item: SortableItem> {
    //points: Vec<Item>,
    bintree: BinTree<Item, GroupDataInner<Item>>,
    params: Params,
}

struct GroupDataInner<Item: SortableItem> {
    lowest_time: Item::Time,
    greatest_time: Item::Time,
}

/// An item that can be sorted using the geogroup algorithm
pub trait SortableItem: Point {
    type Time: Ord;

    fn get_time(&self) -> Self::Time;
}

pub trait GetTime {}

/*
impl<Item: SortableItem> hiearchy::Lazy for DeepSorter<Item> {
    type GroupRef<'a>
        = GroupRef<'a, Item>
    where
        Item: 'a;
    type LeafRef<'a> = usize;
    type GroupMetadata<'a>
        = GroupMetadata
    where
        Self: 'a;
    type NodeMetadata<'a>
        = ()
    where
        Self: 'a;
    type LeafMetadata<'a>
        = ()
    where
        Self: 'a;
    type StructureErr = Infallible;
    type DoesLoading = Infallible;

    fn root(&self) -> GroupRef<'_, Item> {
        GroupRef(&self.points[..])
    }
}
*/

/// Assumes `points` are sorted
/*pub fn sort_to_bintree<Item: SortableItem, It>(points: It) -> BinTree<Item, GroupDataInner<Item>>
where
    for<'a> &'a It: IntoIterator<Item = Item>,
{
    let distances = points
        .into_iter()
        .zip(points.into_iter().skip(1))
        .map(|(prev, next)| prev.distance(&next));

    todo!()
}*/

impl<Item: SortableItem> DeepSorter<Item> {
    pub fn new(mut points: Vec<Item>, params: Params) -> Self {
        points.sort_unstable_by_key(|it| it.get_time());

        Self {
            bintree: todo!(),
            params,
        }
    }

    pub fn add_items(&mut self, item: impl IntoIterator<Item = Item>) {}

    /// Get the smallest group which still encapsulates a given time
    fn get_time_group_mut(
        &mut self,
        time: &Item::Time,
    ) -> Option<&mut BTInnerNode<Item, GroupDataInner<Item>>> {
        //get_time_group_mut(&mut self.bintree, time)

        todo!()
    }
}

/*fn get_time_end<Item: deep_sorter::SortableItem>(tree: &BTInnerNode<Item, GroupDataInner<Item>>) {
    match
}*/

/// Returns [None] if the `tree` itself is out of range, that is if there are either
/// no newer items than `time` OR if there are no older items than `time`.
///
/// This means that for `BinTree::Leaf` we always return [None]. May panic on invalid tree,
/// eg. invalid values
/*fn get_time_group_mut<'a, Item: SortableItem>(
    tree: &'a mut BinTree<Item, GroupDataInner<Item>>,
    time: &Item::Time,
) -> Option<&'a mut BTInnerNode<Item, GroupDataInner<Item>>> {
    match tree {
        BinTree::Leaf(_) => None,
        BinTree::InnerNode(node) => {
            if node.data.lowest_time > *time || node.data.greatest_time < *time {
                return None;
            }

            let [left_child, right_child] = &mut *node.children;
            let left_res = get_time_group_mut(left_child, time);

            match left_res {
                Some(l) => Some(l),
                None => {
                    let right_res = get_time_group_mut(right_child, time);
                    match right_res {
                        Some(r) => Some(r),
                        None => Some(node),
                    }
                }
            }
        }
    }
}*/

pub struct GroupRef<'a, Item: SortableItem>(&'a [Item]);

#[non_exhaustive]
pub struct GroupMetadata {
    pub strength: u8,
}
/*
impl<'a, Item: SortableItem + 'a> hiearchy::lazy::GroupRef<'a> for GroupRef<'a, Item> {
    type Hiearchy = DeepSorter<Item>;
    fn get_children<'b>(
        &self,
    ) -> hiearchy::lazy::LoadingResult<
        impl Iterator<Item = hiearchy::lazy::NodeRef<'b, Self::Hiearchy>>,
        <Self::Hiearchy as hiearchy::Lazy>::StructureErr,
        <Self::Hiearchy as hiearchy::Lazy>::DoesLoading,
    >
    where
        'a: 'b,
    {
        todo!()
    }

    fn group_metadata<'b>(&self) -> <Self::Hiearchy as hiearchy::Lazy>::GroupMetadata<'b>
    where
        'a: 'b,
    {
        todo!()
    }

    fn node_metadata<'b>(&self) -> <Self::Hiearchy as hiearchy::Lazy>::NodeMetadata<'b>
    where
        'a: 'b,
    {
        todo!()
    }
}
*/
