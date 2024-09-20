use std::path::Path;

pub use geogroup_algo as algorithm;
pub use geogroup_common::*;
pub use geogroup_loaders as loaders;
use loaders::DataLoader as _;

/// Sort photos from filesystem and return the resulting the [hiearchy](HiearchyItem)
pub fn sort_from_fs_to_mem(path: &Path) -> HiearchyItem<&Path> {
    let mut data: Vec<_> = path
        .ancestors()
        .map(|file| {
            let loc_data = loaders::GeneralLoader.get_data(file)?;
            Ok((file, loc_data.location?, loc_data.time?))
        })
        .filter_map(|it: Result<_, Box<dyn std::error::Error>>| it.ok()) // TODO: Handle it better, don't just silently ignore failures
        .collect();

    // Ignore the second time for now (until algorithm supports it)
    data.sort_by_key(|(_, _, date)| date[0]);
    let data: Vec<_> = data
        .into_iter()
        // Just convert the rect to a concrete location for now when the algo doesn't support rects
        // TODO: Error on too large rects (ie. inaccurate positions)
        .map(|(file, rect, _)| (rect.center(), file))
        .collect();

    algorithm::sort(data, algorithm::Params::default()).map_leafs(&|(_, f)| f)
}
