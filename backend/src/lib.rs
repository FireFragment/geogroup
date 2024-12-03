use std::path::{Path, PathBuf};

pub use geogroup_algo as algorithm;
pub use geogroup_common::*;
pub use geogroup_loaders as loaders;
pub use geogroup_naming as naming;
pub use geo as geo_lib;
use loaders::DataLoader as _;

/// Sort photos from filesystem and return the resulting the [hiearchy](HiearchyItem)
pub fn sort_from_fs_to_mem(path: &Path) -> HiearchyItem<(geo::Point, PathBuf)> {
    let mut data: Vec<_> = path
        .read_dir()
        .expect("Not a dir") // TODO: Handle
        .flatten() // Maybe this could also be handled better instead of flattening all the options
        .map(|file| {
            let loc_data = loaders::GeneralLoader.get_data(&file.path())?;
            Ok((file.path(), loc_data.location?, loc_data.time?))
        })
        .filter_map(|it: Result<_, Box<dyn std::error::Error>>| it.ok()) // TODO: Handle it better, don't just silently ignore failures
        .collect();

    // Ignore the second time for now (until algorithm supports it)
    data.sort_by_key(|(_, _, date)| date[0]);
    let data: Vec<_> = data
        .into_iter()
        // Just convert the rect to a concrete location for now when the algo doesn't support rects
        // TODO: Error on too large rects (ie. inaccurate positions)
        .map(|(file, rect, _)| (rect.center().into(), file))
        .collect();

    algorithm::sort(data, algorithm::Params::default())
}

pub fn load_directory(path: &Path) -> std::io::Result<HiearchyItem<PathBuf, PathBuf>> {
    if path.is_dir() {
        Ok(HiearchyItem::Group(
            path.read_dir()?
                .map(|file| load_directory(&file?.path())) // TODO: Maybe single failed files shouldn't fail the entire function
                .collect::<Result<_, _>>()?,
            path.to_owned(),
        ))
    } else {
        Ok(HiearchyItem::Item(path.to_owned()))
    }
}
