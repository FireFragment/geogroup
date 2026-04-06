use clap::Parser as _;
use console::Term;
use flatgeobuf::geozero::GeomProcessor;
use flatgeobuf::geozero::PropertyProcessor;
use flatgeobuf::geozero::FeatureProcessor;
use geo::Area;
use geo::GeodesicArea;
use geo::LineString;
use geo::Polygon;
use indicatif::ProgressBar;
use indicatif::ProgressIterator;
use rayon::iter::Either;
use std::cmp::max;
use std::collections::BTreeSet;
use std::error::Error;
use std::io::stdout;
use std::io::Write;
use std::panic;
use std::path::PathBuf;
use std::thread;
use std::thread::sleep;
use std::{collections::{HashMap, HashSet}, env, fs::File, io::Read, sync::{atomic::AtomicU32, mpsc}, time::{Duration, Instant}};

use flatgeobuf::FgbWriter;
use geo::{Coord, Winding};
use kv::Key;
use osmpbfreader::{osmformat::relation, OsmId, OsmPbfReader};
use rayon::iter::{ParallelBridge, ParallelIterator};


/// Copied from osmpbfreader
pub fn get_objs_and_deps_store<R: std::io::Read, F, T>(osm_pbf_reader: &mut OsmPbfReader<R>, mut pred: F, objects: &mut T)
where
    R: std::io::Seek,
    F: FnMut(&osmpbfreader::OsmObj) -> bool,
    T: osmpbfreader::StoreObjs,
{
    use osmpbfreader::*;

    let mut finished = false;
    let mut deps = BTreeSet::new();
    let mut first_pass = true;
    let mut idx =  0;
    let mut failed_objs = 0;
    while !finished {
        log::debug!(target: "nametiles_generator::get_objs_and_deps", "Pass {idx}, first_pass={first_pass}, deps.len()={}", deps.len());
        osm_pbf_reader.rewind().expect("Rewind failed");
        finished = true;

        let mut last_update = Instant::now();

        //let progress = ProgressBar::no_length();
        for (idx, obj) in osm_pbf_reader.par_iter().enumerate() {
            //println!("");

            if last_update.elapsed() > Duration::from_millis(500) {
                let mut stdout = stdout();
                print!("\rProcessed {idx} objects...");
                // or
                // stdout.write(format!("\rProcessing {}%...", i).as_bytes()).unwrap();
                stdout.flush().unwrap();

                last_update = Instant::now();
            }

            let obj = match obj {
                Ok(o) => o,
                Err(err) => {
                    if failed_objs < 16 {
                        log::debug!("Failed to read object: {err}\n{err:?}\n{:?}", err.source());
                    }
                    //panic!("Failed to read object: {err}\n{err:?}\n{:?}", err.source());
                    failed_objs += 1;
                    continue;
                }
            };

            if obj.is_way() {

                //log::debug!("Object {:?} read successfully", obj.id());
            }

            if (!first_pass || !pred(&obj)) && !deps.contains(&obj.id()) {
                continue;
            }
            finished = match obj {
                OsmObj::Relation(ref rel) => rel
                    .refs
                    .iter()
                    .filter(|r| !objects.contains_key(&r.member))
                    .fold(finished, |accu, r| !deps.insert(r.member) && accu),
                OsmObj::Way(ref way) => way
                    .nodes
                    .iter()
                    .filter(|n| !objects.contains_key(&(**n).into()))
                    .fold(finished, |accu, n| !deps.insert((*n).into()) && accu),
                OsmObj::Node(_) => finished,
            };
            deps.remove(&obj.id());
            objects.insert(obj.id(), obj);
        }
        println!(); // To not mess up the console after showing progress

        first_pass = false;
        idx += 1;
    }

    if failed_objs > 0 {
        log::warn!("Failed to read {failed_objs} OSM objects!");
    }
}


struct KvStoreOSM(pub kv::Store);
impl KvStoreOSM {
    pub fn osm_elems_bucket(&self) -> kv::Bucket<Vec<u8>, kv::Bincode<osmpbfreader::OsmObj>> {
        self.0.bucket(None).unwrap()
    }
}

pub fn osm_id_to_kv_key(id: &OsmId) -> Vec<u8> {
    let mut it: Vec<_> = kv::Integer::from(id.inner_id() as u64)
        .to_raw_key().unwrap().bytes()
        .map(|b| b.unwrap()).collect();
    it.push(match id {
        OsmId::Node(_) => 0,
        OsmId::Way(_) => 1,
        OsmId::Relation(_) => 2,
    });
    it
}

impl osmpbfreader::StoreObjs for KvStoreOSM {
    fn insert(&mut self, key: OsmId, value: osmpbfreader::OsmObj) {
        let t = Instant::now();
        let osm_elems_bucket = self.osm_elems_bucket();
        osm_elems_bucket.set(&osm_id_to_kv_key(&key), &kv::Bincode(value)).unwrap();
        log::trace!(
            target: "nametiles_generator::storeobjs_impl",
            "Wrote element {key:?} to the database (it now has {} elements), took {:?}",
            osm_elems_bucket.len(), t.elapsed()
        );
    }

    fn contains_key(&self, key: &OsmId) -> bool {
        //log::trace!(target: "osmpbfreader_StoreObjs", "Query asked for {key:?}");
        self.osm_elems_bucket().contains(&osm_id_to_kv_key(&key)).unwrap()
    }
}

fn get_name(tags: &osmpbfreader::Tags) -> Option<String> {
    tags.get("loc_name").or_else(||
        tags.get("short_name").or_else(||
            tags.get("name")
    )).map(|n| n.to_string())
}

fn cmp_tag<'a>(tags: &osmpbfreader::Tags, key: &str, vals: impl IntoIterator<Item = &'a str>) -> bool {
    vals.into_iter().any(|val|
        tags.get(key).map(|x| x.to_string()).unwrap_or_default() == val
    )
}

fn include_in_tiles(obj: &osmpbfreader::OsmObj) -> bool {
    get_name(obj.tags()).is_some() // Only accept objects with a name
    && (
        (obj.is_relation() && cmp_tag(obj.tags(), "type", ["boundary"])) // Accept all boundary relations
            || (
                // Accept ways and multipolygons with "tags of interest"
                // Important! Update also `include_way_in_tiles` when updating this
                has_tags_of_interest(obj.tags()) && (
                    obj.way().is_some_and(|way| way.is_closed()) ||
                        (obj.is_relation() && cmp_tag(obj.tags(), "type", ["multipolygon"]))
                )
            )
    )
}

fn has_tags_of_interest(tags: &osmpbfreader::Tags) -> bool {
    cmp_tag(tags, "leisure", [
        "bathing_place",
        "beach_resort",
        "garden",
        "golf_course",
        "high_ropes_course",
        "horse_riding",
        "ice_rink",
        "marina",
        "miniature_golf",
        "nature_reserve",
        "outdoor_seating",
        "park",
        "pitch",
        "playground",
        "resort",
        "sports_centre",
        "sports_hall",
        "summer_camp",
        "swimming_area",
        "trampoline_park",
        "water_park",
    ])
    || tags.contains_key("natural")
    || tags.contains_key("landuse")
}


/// Nametiles generator
///
/// Generates a flatgeobuffers file which can be then converted to nametiles using tippecanoe
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path of the .osm.pbf file. Get one at https://www.geofabrik.de/data/download.html
    ///
    /// If skipped, already existing data in cache will be reused.
    input: Option<PathBuf>,
}

fn main() {
    //env_logger::init_from_env(env_logger::Env::new().default_filter_or("info,nametiles_generator=debug"));

     let logger =
         env_logger::Builder::from_env(env_logger::Env::new()
             .default_filter_or("info,nametiles_generator=debug"))
             .init(); // .build();

    let indicatif_progress = indicatif::MultiProgress::new();

        /*indicatif_log_bridge::LogWrapper::new(indicatif_progress.clone(), logger)
         .try_init()
         .unwrap();*/


    let args = Args::parse();

    log::info!("Started");
    log::trace!("Trace-level logging works :)");

    log::info!("Opening the database...");

    struct RelMember {
        way_id: i64,
        rel_id: i64
    }
    //let mut last_logged = Instant::now();
    //
    //
    let temp_db_dir = env::var("NAMETILES_GEN_TEMP_DB").expect("Please specify NAMETILES_GEN_TEMP_DB where I can save temporary data");
    let mut tmp_db = KvStoreOSM(kv::Store::new(kv::Config::new(&temp_db_dir)).unwrap());

    if !tmp_db.osm_elems_bucket().is_empty() {
        log::warn!("There's some already cached data in {temp_db_dir}.
If you use a different source PBF file than before, make sure to clear the cache first.");
    }


    log::info!("Database opened");

    let have_pbf_path = args.input.is_some();
    if let Some(pbf_path) = args.input {

        log::info!("Creating OsmPbfReader...");
        let mut pbf_reader = OsmPbfReader::new(File::open(pbf_path).unwrap());
        log::info!("OsmPbfReader created");

        log::info!("Finding all boundary relations and their members...");
        /*log::info!("Note: this may take a while without any feedback.
If you want to make sure it actually progresses, you can set the environment variable RUST_LOG=info,nametiles_generator=trace
(but even then, it takes some time before the first logs appear)"); // No longer with the progress indicator */
        get_objs_and_deps_store(&mut pbf_reader, include_in_tiles, &mut tmp_db);

        log::info!("...done");
    } else {
        log::warn!("PBF path not provided - using already cached element database
If you intended to provide it, please specify the PBF file as a command line argument.
You can get it for your region from https://download.geofabrik.de/");
    }

    let objs_of_interest_bucket = tmp_db.osm_elems_bucket();

    if !have_pbf_path && objs_of_interest_bucket.is_empty() {
        log::error!("No PBF path provided, and no elements were found in the cache.
This is likely not what you want, the resulting FGB file will likely be empty.
Please provide the PBF file as a command line argument.");
    }

    //log::info!("Total of {} elements were extracted from the PBF file.", objs_of_interest_bucket.len());
    log::info!("Setting up kv database iterator for processing relations...");

    let objs_of_interest = objs_of_interest_bucket.iter()
        .progress_with(indicatif_progress.add(ProgressBar::no_length(
            // `objs_of_interest_bucket.len` takes linear time (https://github.com/zshipko/rust-kv/issues/42),
            // so we can't know the amount of elements beforehand
        )))
        .map(|it| it.unwrap().value::<kv::Bincode<_>>().unwrap().0)
        .filter(include_in_tiles);
        //.filter_map(|obj: kv::Bincode<osmpbfreader::OsmObj>| obj.0.relation().cloned())


    struct ProcessedBoundary {
        pub polygon: Polygon,
        pub name: String
    }

    let obj_processing_results = objs_of_interest.par_bridge().map(|obj| {
        log::trace!("Processing object {:?}", obj.id());

        let Some(obj_name) = get_name(obj.tags()) else {
            log::error!("Object {:?} has no name, but it should have. This is a bug", obj.id());
            return Result::Err(());
        };

        match obj {
            osmpbfreader::OsmObj::Relation(relation) => {

                let opt_unused_member_ways = relation.refs.iter()
                    .filter(|obj|
                        obj.member.is_way()
                            && obj.role == "outer" // For now, we ignore enclaves and inner ways
                    )
                    .map(|way| objs_of_interest_bucket.get(&osm_id_to_kv_key(&way.member)).unwrap().map(|w| w.0.way().unwrap().to_owned()))
                    .collect::<Option<Vec<_>>>();
                let Some(mut unused_member_ways) = opt_unused_member_ways else {
                    log::trace!("Relation {} ({}) is incomplete, skipping it",
                        relation.id.0, obj_name // We filtered the relations to all contain names
                    );
                    return Result::Err(());//continue 'relation_loop;
                };

                //let way_info = HashMap::new();
                //
                let mut polygons = Vec::new();

                while !unused_member_ways.is_empty() { // This loop iterates over all outer rings of the polygon
                    let Some(first_way) = unused_member_ways.pop() else {
                        log::trace!("Relation {} ({}) contains no ways, skipping it",
                            relation.id.0, obj_name // We filtered the relations to all contain names
                        );
                        return Result::Err(());//continue 'relation_loop;
                    };

                    let Some((mut first_node, mut current_node)) = first_and_last_node(&objs_of_interest_bucket, &first_way) else {
                        log::trace!("Relation {} ({}) contains an incomplete way ({}), skipping it",
                            relation.id.0,
                            obj_name, // We filtered the relations to all contain names
                            first_way.id.0
                        );
                        return Result::Err(());// 'relation_loop;
                    };

                    let mut polygon_nodes: Vec<_> = osm_way_to_coords(&objs_of_interest_bucket, first_way).collect();

                    while first_node.id != current_node.id {
                        struct WayInRelInfo {
                            /// Index in `unused_member_ways`
                            way_idx: usize,
                            reversed: bool
                        }

                        let next_way_info = unused_member_ways.iter().enumerate().find_map(|(idx, way)| {
                            let Some((first_node_of_way, last_node_of_way)) = first_and_last_node(&objs_of_interest_bucket, &way) else {
                                log::debug!("Relation {} ({}) contains an incomplete way ({}), it will likely be skipped and marked as unclosed relation",
                                    relation.id.0,
                                    obj_name, // We filtered the relations to all contain names
                                    way.id.0
                                );
                                return None;
                            };

                            if first_node_of_way.id == current_node.id {
                                current_node = last_node_of_way;
                                Some(WayInRelInfo {
                                    way_idx: idx, reversed: false
                                })
                            } else if last_node_of_way.id == current_node.id {
                                current_node = first_node_of_way;
                                Some(WayInRelInfo {
                                    way_idx: idx, reversed: true
                                })
                            } else { None }
                        });

                        let Some(next_way_info) = next_way_info else {
                            // TODO: Don't panic
                            log::debug!("Unclosed relation: {} ({}) - node {} belongs to end of only one way, skipping it",
                                relation.id.0,
                                obj_name,
                                current_node.id.0
                            ); // We filtered the relations to all contain names
                            return Result::Err(()); //continue 'relation_loop;
                        };

                        let mut found_way = unused_member_ways.swap_remove(next_way_info.way_idx);
                        if next_way_info.reversed {
                            found_way.nodes.reverse();
                        }

                        let nodes = osm_way_to_coords(&objs_of_interest_bucket, found_way);

                        polygon_nodes.extend(nodes);
                    }
                    let mut nodes_ls = LineString::from(polygon_nodes);
                    nodes_ls.make_ccw_winding();  // For correct area calculations later
                    polygons.push(geo::Polygon::new(
                        nodes_ls,
                        Vec::new()
                    ));
                };

                /*let Some(winding_order) = geo::LineString::from(polygon_nodes).winding_order() else {
                    log::warn!("Relation {} ({}) contains less than 3 distinct coordinates, skipping it",
                        relation.id.0,
                        relation_name
                    );
                    return Result::Err(());
                };*/



                /*let poly = polyline::encode_coordinates(polygon_nodes, 5).unwrap();
                println!("");
                println!("{}", relation_name);
                println!("{poly}");*/

                // Warning: We rely below on the fact that there are NO more than 1 exterior and no interior rings
                Result::Ok(Either::Left(polygons.into_iter().map(move |polygon| ProcessedBoundary {
                    polygon,
                    name: obj_name.clone(),
                }).par_bridge()))
            }
            osmpbfreader::OsmObj::Way(way) => {
                let mut nodes_ls: LineString = osm_way_to_coords(&objs_of_interest_bucket, way).collect();
                nodes_ls.make_ccw_winding(); // For correct area calculations later
                Ok(Either::Right(rayon::iter::once(ProcessedBoundary {
                    polygon:
                        geo::Polygon::new(nodes_ls, Vec::new()),
                    name: obj_name
                })))
            }
            osmpbfreader::OsmObj::Node(_) => {
                // We don't process nodes here
                Err(())
            }
        }
    })
    .filter_map(|res| res.ok()) // TODO: Don't ignore failures
    .flatten();

    log::info!("Initializing the FGB dataset");

    let (polygons_tx, polygons_rx) = mpsc::channel::<ProcessedBoundary>();

    let mut fgb_writer = FgbWriter::create("nametiles_base", flatgeobuf::GeometryType::MultiPolygon).unwrap();
    fgb_writer.dataset_begin(Some("nametiles_iter2")).unwrap();

    fgb_writer.add_column("name", flatgeobuf::ColumnType::String, |_fbb, col| {
        col.nullable = false;
    });
    fgb_writer.add_column("area", flatgeobuf::ColumnType::ULong, |_fbb, col| {
        col.nullable = false;
    });

    log::info!("Initialized the FGB dataset");


    //let fgb_writing_ref = prog_fgb_writing.clone();
    let writign_thread_handle = thread::spawn(move || {
        let mut max_area = 0;

        log::info!("Collecting FGB dataset...");
        for (idx, ProcessedBoundary { polygon, name }) in polygons_rx.iter().enumerate() {
            //fgb_writing_ref.inc(idx as u64);
            //
            fgb_writer.feature_begin(idx as u64).unwrap();
            fgb_writer.properties_begin().unwrap();
            fgb_writer.property(0, "name", &geozero::ColumnValue::String(&name)).unwrap();

            // TODO: Would be more performant to calculate this on the sending threads
            // (multithreaded)
            let area = polygon.geodesic_area_unsigned() as u64;
            max_area = max(max_area, area);
            fgb_writer.property(1, "area", &geozero::ColumnValue::ULong(area)).unwrap();
            //fgb_writer.property();
            fgb_writer.properties_end().unwrap();
            fgb_writer.geometry_begin().unwrap();
            fgb_writer.polygon_begin(true, 1, 0).unwrap();
            fgb_writer.linestring_begin(false, polygon.exterior().0.len(), 0).unwrap();
            for (idx, coord) in polygon.exterior().0.iter().enumerate() {
                fgb_writer.xy(coord.x, coord.y, idx).unwrap();
            }
            fgb_writer.linestring_end(false, 0).unwrap();
            fgb_writer.polygon_end(true, 0).unwrap();

            fgb_writer.geometry_end().unwrap();

            fgb_writer.feature_end(idx as u64).unwrap();
        }


        log::info!("All FGB items added, finishing dataset...");
        fgb_writer.dataset_end().unwrap();

        log::info!("FGB dataset finished, writing it to file...");

        fgb_writer.write(File::create("dataset.fgb").unwrap()).unwrap();
        log::info!("FGB dataset written");

        max_area
    });

    obj_processing_results
        .for_each(|boundary| {
            //prog_fgb_writing.inc_length(1);
            polygons_tx.send(boundary).unwrap();
                //println!("Sending..");
        });

    drop(polygons_tx); // Stop the polygon writing thread

    let max_area = writign_thread_handle.join().unwrap();
    log::debug!("The biggest area is {max_area}m2 (this is a quick sanity check for area calculations)");

        /*let (success, failed) = relation_processing_results.map(|res| {
        match res {
            Ok(_) => (1, 0),
            Err(()) => (0, 1),
        }
    }).reduce(|| (0, 0), |(a1, a2), (b1, b2)| (a1 + b1, a2 + b2));

    log::info!("{success} relations scanned successfully, {failed} relations failed");*/

    /*let ways = objs_of_interest.values()
        .filter_map(|obj| obj.relation())
        .flat_map(|relation| relation.refs.iter())
        .filter(|rf| rf.member.is_way()) // TODO: Check for role
        .map(|rf| rf.member.inner_id());*/


    /*
    // Ways that are part of boundaries
    let ways_of_interest: HashSet<_> = pbf_reader.par_iter_relations().map(|res| res.unwrap()).filter(|relation| {
            relation.tags.get("type").map(|x| x.to_string()) == Some("boundary".into())
                && relation.tags.get("name").is_some()
        })
        .map(|relation| relation.refs.into_iter()
            .filter(|rf| rf.member.is_way()) // TODO: Check for role
            .map(|rf| rf.member.inner_id())
        //.map(move |rf| RelMember { rel_id: relation.id.0, way_id: rf.member.inner_id() })
        )
        .flatten()
        .enumerate()
        .inspect(|(idx, _)| {
            let now = Instant::now();
            if now - last_logged > Duration::from_millis(10) {
                log::info!("Finding, found {idx} ways");
                last_logged = now;
            }
        })
        .map(|(_, way_id)| way_id)
        .collect();

    log::info!("...done - found {} ways.", ways_of_interest.len());

    log::info!("Writing geometry of the {} ways to the temporary cache...", ways_of_interest.len());

    /*pbf_reader.

    pbf_reader.par_iter_ways()
        .map(|way| way.unwrap())
        .filter(|way| ways_of_interest.contains(&way.id.0))
        .map(|way| geo_types::LineString::from(way.nodes.into_iter().map(|node| node.0)));

    let tmp_db = kv::Store::new(kv::Config::new("/nix/temporary/nametilesgen/db"));*/
    */

    /*for (idx, way_id) in ways_of_interest.enumerate() {


        // Logging
    }*/





}

fn osm_way_to_coords(objs_of_interest_bucket: &kv::Bucket<'_, Vec<u8>, kv::Bincode<osmpbfreader::OsmObj>>, found_way: osmpbfreader::Way) -> impl Iterator<Item = Coord> {

    found_way.nodes.into_iter().flat_map(|node|
        {
            //println!(");
            let Some(bc) = objs_of_interest_bucket
                .get(&osm_id_to_kv_key(&OsmId::Node(node)))
                .unwrap() else {
                    log::warn!("Node {node:?} not found in db");
                    return None;
                };
            let node = bc.0.node().unwrap();
            Some(Coord {
                x: node.lon(),
                y: node.lat(),
            })
        }
    )
}

fn first_and_last_node(objs_of_interest_bucket: &kv::Bucket<'_, Vec<u8>, kv::Bincode<osmpbfreader::OsmObj>>, way: &osmpbfreader::Way) -> Option<(osmpbfreader::Node, osmpbfreader::Node)> {
    let first_node = objs_of_interest_bucket
        .get(&osm_id_to_kv_key(&OsmId::Node(way.nodes[0])))
        .unwrap()? // Question mark for cases when the node is missing but the way contains a reference to it
        .0.node()
        .unwrap().to_owned();

    let last_node = objs_of_interest_bucket.get(&osm_id_to_kv_key(&OsmId::Node(*way.nodes.last().unwrap())))
        .unwrap()? // Question mark for cases when the node is missing but the way contains a reference to it
        .0.node()
        .unwrap().to_owned();

    Some((first_node, last_node))
}
