//! The logic of naming a [hiearchy](HiearchyItem)

use address_formatter::Place;
use geocoding::Point;
use geogroup_common::HiearchyItem;

use crate::*;

type PlaceAnnotatedHiearchy<LeafData> = HiearchyItem<(Place, LeafData), Place>;

fn get_place_of_hiearchy<LeafData>(hiearchy: &PlaceAnnotatedHiearchy<LeafData>) -> &Place {
    match hiearchy {
        HiearchyItem::Group(_, pl) => pl,
        HiearchyItem::Item((pl, _)) => pl,
    }
}

/// [`Place`] doesn't implement [`Clone`] for some reason...
pub fn place_to_owned(pl: &Place) -> Place {
    Place::from(
        pl.iter()
            .filter_map(|(k, v)| v.as_ref().map(|v| (k, v.as_str()))),
    )
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

impl RevGeocoder<'_> {
    pub fn name_hiearchy<LeafData>(
        &self,
        hiearchy: HiearchyItem<(Point, LeafData)>,
    ) -> HiearchyItem<(String, LeafData), String> {
        self.annotate_hiearchy_with_full_places(hiearchy)
            .map_leafs(&|(place, data)| (format_place(place), data))
            .map_group_data(&format_place)
    }

    /// This will annotate the with _full_ places, meaning that it will duplicate parents' fields in children's places, such as:
    ///
    /// ```text
    /// CITY => Rome
    ///   ->
    /// ```
    fn annotate_hiearchy_with_full_places<LeafData>(
        &self,
        hiearchy: HiearchyItem<(Point, LeafData)>,
    ) -> PlaceAnnotatedHiearchy<LeafData> {
        match hiearchy {
            HiearchyItem::Group(children, data) => {
                let named_children: Vec<_> = children
                    .into_iter()
                    .map(|child| self.annotate_hiearchy_with_full_places(child))
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
            HiearchyItem::Item((point, data)) => {
                HiearchyItem::Item((
                    build_place(
                        self.reverse_geocode(&point)
                            .expect("Failed to geocode") /*TODO: Handle*/
                            .iter()
                            .map(|(k, v)| {
                                (
                                    k.as_str(),
                                    v.to_owned(), /* TODO: Suboptimal performance */
                                )
                            }),
                    ),
                    data,
                ))
            }
        }
    }
}
