use std::{
    env,
    path::{Path, PathBuf},
    rc::Rc,
};

use bytes::Bytes;
use geo::{Contains, Coord};
use mvt_reader::feature::Value as TileValue;
use pmtiles::{PmtError, TileCoord, TileId, s3};
use thiserror::Error;

pub struct NametilesConnection {
    pmtiles: DynPmTilesReader,
    cache_dir: Option<PathBuf>,
}

impl From<pmtiles::AsyncPmTilesReader<pmtiles::S3Backend>> for NametilesConnection {
    fn from(value: pmtiles::AsyncPmTilesReader<pmtiles::S3Backend>) -> Self {
        NametilesConnection {
            pmtiles: DynPmTilesReader::S3(value),
            cache_dir: nametiles_default_cache_dir(),
        }
    }
}

/// `AsyncPmTilesReader` which can use one of multiple backends
enum DynPmTilesReader {
    Mmap(pmtiles::AsyncPmTilesReader<pmtiles::MmapBackend>),
    S3(pmtiles::AsyncPmTilesReader<pmtiles::S3Backend>),
}

impl DynPmTilesReader {
    pub async fn get_tile_decompressed<Id: Into<TileId>>(
        &self,
        tile_id: Id,
    ) -> pmtiles::PmtResult<Option<bytes::Bytes>> {
        match self {
            Self::Mmap(reader) => reader.get_tile_decompressed(tile_id).await,
            Self::S3(reader) => reader.get_tile_decompressed(tile_id).await,
        }
    }
}

impl From<pmtiles::AsyncPmTilesReader<pmtiles::MmapBackend>> for DynPmTilesReader {
    fn from(value: pmtiles::AsyncPmTilesReader<pmtiles::MmapBackend>) -> Self {
        Self::Mmap(value)
    }
}

impl From<pmtiles::AsyncPmTilesReader<pmtiles::S3Backend>> for DynPmTilesReader {
    fn from(value: pmtiles::AsyncPmTilesReader<pmtiles::S3Backend>) -> Self {
        Self::S3(value)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("PMTiles format error: {0}")]
    PmtError(#[from] PmtError),
    #[error("Tile {0:?} is missing from the nametile dataset")]
    TileMissing(pmtiles::TileCoord),
    /// Cast from [`mvt_reader::error::ParserError`]
    #[error("MVT format error: {0}")]
    MvtError(String),
}

#[derive(Error, Debug)]
pub enum NewFromBuckerError {
    #[error(transparent)]
    PmtError(#[from] PmtError),
    #[error("Failed to get S3 bucket credentials from environment variables: {0}")]
    NoEnvCredentials(#[from] s3::creds::error::CredentialsError),
    #[error(transparent)]
    S3Err(#[from] s3::error::S3Error),
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
    MvtError(String),
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

fn nametiles_default_cache_dir() -> Option<PathBuf> {
    Some(
        dirs::cache_dir()
            .or_else(|| {
                log::warn!("Can't obtain system cache directory, not using any cache.");
                None
            })?
            .join("gg_nametiles")
            .join("tiles"),
    )
}

fn env_var_or_default(var_name: &str, default: &str) -> String {
    env::var(var_name).unwrap_or_else(|_| {
        log::info!("{var_name} not found, using default value: {default}");
        default.into()
    })
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NamePart {
    pub name: String,
    /// [`None`] when unknown
    pub area: Option<u64>,
}

impl NametilesConnection {
    pub async fn new_from_file(path: &Path) -> Result<Self, pmtiles::PmtError> {
        Ok(Self {
            pmtiles: pmtiles::AsyncPmTilesReader::new_with_path(path)
                .await?
                .into(),

            cache_dir: None, // We don't need cache when we're already reading from the disk
        })
    }

    pub async fn new_from_env() -> Result<Self, NewFromBuckerError> {
        if let Some(file) = env::var("NAMETILES_FILE").ok() {
            log::info!("Using {file} as nametiles source (as set by the NAMETILES_FILE env var).");
            Ok(Self::new_from_file(Path::new(&file)).await?)
        } else {
            log::info!("NAMETILES_FILE env var not found, using S3 bucket instead.");
            Self::new_from_env_bucket().await
        }
    }

    pub async fn new_from_env_bucket() -> Result<Self, NewFromBuckerError> {
        Ok(Self {
            pmtiles: pmtiles::AsyncPmTilesReader::new_with_bucket_path(
                *pmtiles::s3::Bucket::new(
                    &env_var_or_default("NAMETILES_S3_BUCKET_NAME", "geogroup-nametiles-czechia"),
                    s3::Region::Custom {
                        region: env_var_or_default("NAMETILES_S3_REGION", "eu-central-003"),
                        endpoint: env_var_or_default(
                            "NAMETILES_S3_ENDPOINT",
                            "s3.eu-central-003.backblazeb2.com",
                        ),
                    },
                    s3::creds::Credentials::from_env()?,
                )?,
                env_var_or_default("NAMETILES_S3_BUCKET_FILE", "out.pmtiles"),
            )
            .await?
            .into(),
            cache_dir: nametiles_default_cache_dir(),
        })
    }

    pub async fn get_name(&self, point: geo::Coord) -> Result<Vec<NamePart>, Error> {
        let tilepos = TilePos::new(point, 10);

        Ok(
            mvt_reader::Reader::new(self.get_tile_bytes(tilepos.tile).await?)?
                .get_features(0)?
                .into_iter()
                .filter(|feature| Contains::contains(feature.get_geometry(), &tilepos.coord))
                .flat_map(|feature| {
                    let properties = feature.properties.unwrap_or_default();

                    let TileValue::String(name) = properties.get("name")? else {
                        return None;
                    };

                    Some(NamePart {
                        name: name.to_owned(),
                        area: match properties.get("area") {
                            Some(TileValue::UInt(area)) => Some(*area),
                            Some(TileValue::Int(area) | TileValue::SInt(area)) => Some(*area as u64),
                            Some(TileValue::Float(area)) => Some(*area as u64),
                            Some(TileValue::Double(area)) => Some(*area as u64),
                            Some(TileValue::String(area)) => area.parse().inspect_err(|err|
                                log::debug!("Area is a string which cannot be parsed: {err}\nArea is \"{area}\"")
                            ).ok(),
                            other => {
                                log::debug!(
                                    "Missing or invalid area (it's {other:?})",
                                );
                                None
                            }
                        },
                    })
                })
                .collect(),
        )
    }

    /// Implements caching
    pub async fn get_tile_bytes(&self, tile_pos: pmtiles::TileCoord) -> Result<Vec<u8>, Error> {
        let tile_cache_path = self.cache_dir.as_ref().map(|cache_root| {
            let tile_name = format!("{}-{}-{}", tile_pos.z(), tile_pos.x(), tile_pos.y());
            cache_root.join(tile_name)
        });

        // Try to read tile from cache
        if let Some(cache_path) = &tile_cache_path {
            match tokio::fs::read(&cache_path).await {
                Ok(cached_tile) => return Ok(cached_tile),
                Err(err) => log::debug!("Failed to read cache path {cache_path:?}: {err}"),
            }
        }

        // Try to fetch the tile
        let Some(network_tile) = self.pmtiles.get_tile_decompressed(tile_pos).await? else {
            return Err(Error::TileMissing(tile_pos));
        };

        // Try to write tile to cache
        if let Some(tile_cache_path) = &tile_cache_path {
            if let Some(tile_cache_dir) = tile_cache_path.parent() {
                if let Err(err) = tokio::fs::create_dir_all(tile_cache_dir).await {
                    log::debug!("Failed to create cache directory: {err}\nPath: {tile_cache_dir:?}")
                }
            }

            // TODO: Don't block on cache write
            if let Err(err) = tokio::fs::write(tile_cache_path, &network_tile).await {
                log::debug!(
                    "Failed to write fetched nametile to cache: {err}\nPath: {tile_cache_path:?}"
                )
            };
        }

        Ok(network_tile.into())
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
            coord: Coord {
                x: (x.fract() * TILE_SIZE as f64).floor() as f32,
                y: (y.fract() * TILE_SIZE as f64).floor() as f32,
            },
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
