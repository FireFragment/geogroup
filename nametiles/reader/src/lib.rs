use std::{path::Path, rc::Rc};

use geo::{Contains, Coord};
use pmtiles::{PmtError, TileCoord, TileId};
use thiserror::Error;

pub struct NametilesConnection {
    pmtiles: pmtiles::AsyncPmTilesReader<pmtiles::MmapBackend>,

}

#[derive(Error, Debug)]
pub enum Error {
    #[error("PMTiles format error: {0}")]
    PmtError(#[from] PmtError),
    #[error("Tile {0:?} is missing from the nametile dataset")]
    TileMissing(pmtiles::TileCoord),
    /// Cast from [`mvt_reader::error::ParserError`]
    #[error("MVT format error: {0}")]
    MvtError(String)
}

impl From<mvt_reader::error::ParserError> for Error {
    fn from(value: mvt_reader::error::ParserError) -> Self {
        Self::MvtError(value.to_string())
    }
}

/// Like [`Error`], but also implements [`Clone`]. Suberrors are not represented as they came, but as strings to allow clonability
#[derive(Error, Debug, Clone)]
pub enum SimpleError {
    #[error("PMTiles format error: {0}")]
    PmtError(String),
    #[error("Tile {0:?} is missing from the nametile dataset")]
    TileMissing(pmtiles::TileCoord),
    #[error("MVT format error: {0}")]
    MvtError(String)
}

impl From<Error> for SimpleError {
    fn from(value: Error) -> Self {
        match value {
            Error::PmtError(err) => Self::PmtError(err.to_string()),
            Error::TileMissing(tile) => Self::TileMissing(tile),
            Error::MvtError(err) => Self::MvtError(err.to_string()),
        }
    }
}

impl NametilesConnection {
    pub async fn new_from_file(path: &Path) -> Result<Self, pmtiles::PmtError> {
        Ok(Self {
            pmtiles: pmtiles::AsyncPmTilesReader::new_with_path(path).await?,
        })
    }

    pub async fn get_name(&self, point: geo::Coord) -> Result<Vec<String>, Error> {
        let tilepos = TilePos::new(point, 10);
        let Some(tile) = self.pmtiles.get_tile_decompressed(tilepos.tile).await? else {
            return Err(Error::TileMissing(tilepos.tile));
        };

        Ok(mvt_reader::Reader::new(tile.into())?.get_features(0)?.into_iter().filter(|feature| {
            Contains::contains(feature.get_geometry(), &tilepos.coord)
        }).flat_map(|feature| feature.properties.unwrap_or_default().get("name").and_then(|v| match v {
            mvt_reader::feature::Value::String(str) => Some(str.to_owned()),
            _ => None
        })).collect())
    }
}

pub struct TilePos {
    /// Coordinates of the tile
    pub tile: pmtiles::TileCoord,
    /// Coordinates inside of the tile
    pub coord: Coord<f32>,
}

const TILE_SIZE: u16 = 4096;

impl TilePos {
    pub fn new(point: geo::Coord, zoom: u8) -> Self {
        let (x, y) = project_point_to_tiles(point, zoom);
        Self {
            tile: pmtiles::TileCoord::new(zoom, x.floor() as u32, y.floor() as u32)
                .expect("`project_point_to_tiles` created invalid coordinates"),
            coord: Coord{ x: (x.fract() * TILE_SIZE as f64).floor() as f32, y: (y.fract() * TILE_SIZE as f64).floor() as f32 },

        }
    }
}

/// Returns the tile containing the given point.
///
/// Returns decimal number, which may be helpful to locate the point on the tile
/// To find out the actual tilename, floor the returned coordinates
pub fn project_point_to_tiles(point: geo::Coord, zoom: u8) -> (f64, f64) {
    // Derived from https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames#Implementations
    use std::f64::consts::PI;

    let n = 2u32.pow(zoom as u32) as f64;
    let lat_rad = point.y / 180.0 * PI;
    (
        n * (point.x + 180.0) / 360.0,
        n * (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / PI) / 2.0,
    )
}
