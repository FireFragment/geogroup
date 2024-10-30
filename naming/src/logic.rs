//! The logic of naming a [hiearchy](HiearchyItem)

use address_formatter::Place;
use geocoding::Point;
use geogroup_common::HiearchyItem;

use crate::*;

pub fn name_hiearchy<LeafData>(
    hiearchy: HiearchyItem<(LeafData, Point)>,
) -> HiearchyItem<(LeafData, String), String> {
    annotate_hiearchy_with_full_places(hiearchy)
        .map_leafs(&|(data, place)| (data, format_place(place)))
        .map_group_data(&format_place)
}

type PlaceAnnotatedHiearchy<LeafData> = HiearchyItem<(LeafData, Place), Place>;

fn get_place_of_hiearchy<LeafData>(hiearchy: &PlaceAnnotatedHiearchy<LeafData>) -> &Place {
    match hiearchy {
        HiearchyItem::Group(_, pl) => pl,
        HiearchyItem::Item((_, pl)) => pl,
    }
}

/// [`Place`] doesn't implement [`Clone`] for some reason...
pub fn place_to_owned(pl: &Place) -> Place {
    Place::from(
        pl.iter()
            .filter_map(|(k, v)| v.as_ref().map(|v| (k, v.as_str()))),
    )
}

/// This will annotate the with _full_ places, meaning that it will duplicate parents' fields in children's places, such as:
///
/// ```text
/// CITY => Rome
///   ->
/// ```
fn annotate_hiearchy_with_full_places<LeafData>(
    hiearchy: HiearchyItem<(LeafData, Point)>,
) -> PlaceAnnotatedHiearchy<LeafData> {
    match hiearchy {
        HiearchyItem::Group(children, data) => {
            let named_children: Vec<_> = children
                .into_iter()
                .map(|child| annotate_hiearchy_with_full_places(child))
                .collect();

            if let Some(place_of_self) = named_children
                .iter()
                .map(get_place_of_hiearchy)
                .map(place_to_owned) // TODO: Not the best performance here because of cloning
                .reduce(|a, b| intersection(&a, &b))
            {
                HiearchyItem::Group(named_children, place_of_self)
            } else {
                HiearchyItem::Group(named_children, Place::default())
            }
        }
        HiearchyItem::Item((data, point)) => {
            HiearchyItem::Item((
                data,
                build_place(
                    reverse_geocode(todo!(), todo!(), &point)
                        .expect("Failed to geocode") /*TODO: Handle*/
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.to_owned() /* TODO: Suboptimal performance */)),
                ),
            ))
        }
    }
}

/// Returns [Place] with just fields, where both arguments share the same values
pub fn intersection(a: &Place, b: &Place) -> Place {
    a.iter()
        .filter(|(comp, val)| b[*comp] == **val)
        .filter_map(|(c, v)| {
            if let Some(rv) = v {
                Some((c, rv.as_str()))
            } else {
                None
            }
        })
        .into()
}
