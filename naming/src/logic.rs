//! The logic of naming a [hiearchy](HiearchyItem)

use address_formatter::Place;
use geocoding::{GeocodingError, Point};
use geogroup_common::HiearchyItem;

use crate::*;

type PlaceAnnotatedHiearchy<LeafData> = HiearchyItem<(Place, LeafData), Place>;

fn get_place_of_hiearchy<LeafData>(hiearchy: &PlaceAnnotatedHiearchy<LeafData>) -> &Place {
    match hiearchy {
        HiearchyItem::Group(_, pl) => pl,
        HiearchyItem::Item((pl, _)) => pl,
    }
}

/// Operations on places
pub mod place_op {
    use super::*;

    /// [`Place`] doesn't implement [`Clone`] for some reason...
    pub fn to_owned(pl: &Place) -> Place {
        Place::from(
            pl.iter()
                .filter_map(|(k, v)| v.as_ref().map(|v| (k, v.as_str()))),
        )
    }

    /// Returns [Place] with just fields, where both arguments share the same values
    pub fn intersection(a: &Place, b: &Place) -> Place {
        a.iter()
            .filter(|(comp, val)| b[*comp] == **val)
            .filter_map(|(c, v)| v.as_ref().map(|rv| (c, rv.as_str())))
            .into()
    }

    /// Removes the fields present in the second argument from the first argument
    pub fn subtract(a: &mut Place, b: &Place) {
        for (component, _) in b.iter().filter(|(_, val)| val.is_some()) {
            a[component] = None;
        }
    }
}

/// This will prune places in the [annotated hiearchy](PlaceAnnotatedHiearchy) so that they don't duplicate information already present in parent groups place
/// Eg. if there is an item with place `Rome, Piazza del Colloseo` under a group `Rome`, it will remove `Rome` from the items place
/// and only keep `Piazza del Colloseo`, so that the information about `Rome` is not duplicated between the item and its parent.
fn prune_hiearchy_places<LeafData>(
    hiearchy: &mut PlaceAnnotatedHiearchy<LeafData>,
    parent_place: &Place,
) {
    match hiearchy {
        HiearchyItem::Item((place, _)) => place_op::subtract(place, parent_place),
        HiearchyItem::Group(children, place) => {
            for child in children {
                prune_hiearchy_places(child, place)
            }

            place_op::subtract(place, parent_place)
        }
    }
}

impl RevGeocoder<'_> {
    /// Fetch places of all points, saving them to cache for faster future naming
    pub fn prefetch_places(
        &self,
        points: &Vec<Point>,
        mut report_progress: impl FnMut(PrefetchProgressReport),
    ) -> Result<(), GeocodingError> {
        let total_points = points.len();

        for (idx, point) in points.into_iter().enumerate() {
            report_progress(PrefetchProgressReport {
                done: idx,
                total: total_points,
            });
            self.reverse_geocode(point)?;
        }

        Ok(())
    }

    pub fn name_hiearchy<LeafData>(
        &self,
        hiearchy: HiearchyItem<(Point, LeafData)>,
    ) -> HiearchyItem<(String, LeafData), String> {
        // 1. Annotate
        let mut annotated_hiearchy = self.annotate_hiearchy_with_full_places(hiearchy);
        // 2. Prune
        prune_hiearchy_places(&mut annotated_hiearchy, &Place::default());
        // 3. Format
        annotated_hiearchy
            .map_leafs(&|(place, data)| (format_place(place), data))
            .map_group_data(&format_place)
    }

    /// This will annotate the with _full_ places, meaning that it will duplicate parents' fields in children's places, such as:
    ///
    /// ```text
    /// CITY => Rome
    ///   -> CITY => Rome, STREET => Piazza del Colloseo
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
                    .map(place_op::to_owned) // TODO: Not the best performance here because of cloning
                    .reduce(|a, b| place_op::intersection(&a, &b))
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
