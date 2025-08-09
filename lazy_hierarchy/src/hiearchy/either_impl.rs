//! [`GroupRef`] is implemented for [`Either`] if both sides' implementation of groupref has the same
//! [`GroupData`](GroupRef::GroupData),
//! [`LeafData`](GroupRef::LeafData),
//! [`NodeData`](GroupRef::NodeData), and
//! [`StructureErr`](GroupRef::StructureErr).

use super::*;
use either::for_both;

impl<
        Left: GroupRef,
        Right: GroupRef<
            GroupData = Left::GroupData,
            LeafData = Left::LeafData,
            NodeData = Left::NodeData,
            StructureErr = Left::StructureErr,
        >,
    > GroupRef for Either<Left, Right>
{
    type NodeData = Left::NodeData;
    type LeafData = Left::LeafData;
    type GroupData = Left::GroupData;
    type StructureErr = Left::StructureErr;
    type LeafRef = Either<Left::LeafRef, Right::LeafRef>;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        Ok(match self {
            Either::Left(left) => Either::Left(left.get_children()?.map(|child| match child {
                NodeRef::Leaf(leaf) => NodeRef::Leaf(Either::Left(leaf)),
                NodeRef::Group(group) => NodeRef::Group(Either::Left(group)),
            })),
            Either::Right(right) => Either::Right(right.get_children()?.map(|child| match child {
                NodeRef::Leaf(leaf) => NodeRef::Leaf(Either::Right(leaf)),
                NodeRef::Group(group) => NodeRef::Group(Either::Right(group)),
            })),
        })
    }

    fn group_data(&self) -> Self::GroupData {
        for_both!(self, s => s.group_data())
    }

    fn node_data(&self) -> Self::NodeData {
        for_both!(self, s => s.node_data())
    }
}

impl<Left: LeafRef, Right: LeafRef<LeafData = Left::LeafData, NodeData = Left::NodeData>> LeafRef
    for Either<Left, Right>
{
    type LeafData = Left::LeafData;
    type NodeData = Left::NodeData;

    fn leaf_data(&self) -> Self::LeafData {
        for_both!(self, s => s.leaf_data())
    }

    fn node_data(&self) -> Self::NodeData {
        for_both!(self, s => s.node_data())
    }
}
