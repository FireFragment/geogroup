//! Utilities for calculating strength of groups

use geogroup_common::hiearchy;
use geogroup_common::hiearchy::lazy::LazyHiearchyUtils as _;

use crate::Distance;

pub fn calc_strength<H: hiearchy::Lazy, DistF: Fn(H::GroupMetadata<'_>) -> Distance>(
    hiearchy: H,
    get_distance: DistF,
) where
    for<'a> H::GroupRef<'a>: Clone,
{
    //hiearchy.with_parent().map_groups(|gr|);
}
