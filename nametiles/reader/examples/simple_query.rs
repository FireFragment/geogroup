use std::path::PathBuf;

use clap::Parser as _;
use geo::Coord;
use itertools::Itertools;
use nametiles_reader::*;

/// This example takes location in format `lat,lon` as an argument and prints all names of the location using nametiles
#[derive(clap::Parser)]
pub struct Args {
    /// Coordinates to name in format `lat,lon`
    coords: String
}

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    let args = Args::parse();

    let zoom = 10;
    let coord = {
        let (y, x) = args.coords.split(",").collect_tuple().expect("Too many coordinates provided");
        Coord {
            y: y.parse().expect("Invalid number"),
            x: x.parse().expect("Invalid number"),
        }
    };
    let (x, y) = project_point_to_tiles(coord, zoom);
    println!("Reading tile {zoom}/{}/{}", x.floor(), y.floor());

    let nametiles = NametilesConnection::new_from_file(&PathBuf::from(
        "/nix/temporary/nametilesgen/out.pmtiles", // TODO: Don't hardcode the path
    ))
    .await
    .unwrap();


    println!("{}", nametiles.get_name(coord).await.unwrap().into_iter().map(|n| n.name).join(", "));
}
