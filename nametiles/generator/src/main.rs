use clap::Parser as _;
use flatgeobuf::geozero::GeomProcessor;
use flatgeobuf::geozero::PropertyProcessor;
use flatgeobuf::geozero::FeatureProcessor;
use geo::LineString;
use std::path::PathBuf;
use std::thread;
use std::{collections::{HashMap, HashSet}, env, fs::File, io::Read, sync::{atomic::AtomicU32, mpsc}, time::{Duration, Instant}};

use flatgeobuf::FgbWriter;
use geo::{Coord, Winding};
use kv::Key;
use osmpbfreader::{osmformat::relation, OsmId, OsmPbfReader};
use rayon::iter::{ParallelBridge, ParallelIterator};


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
        //println!("Wrote something");
        self.osm_elems_bucket().set(&osm_id_to_kv_key(&key), &kv::Bincode(value)).unwrap();
    }

    fn contains_key(&self, key: &OsmId) -> bool {
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
    obj.is_relation()
        && (
            cmp_tag(obj.tags(), "type", ["boundary"])
            || (
                cmp_tag(obj.tags(), "type", ["multipolygon"]) && (
                    cmp_tag(obj.tags(), "leisure", [
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
                    || obj.tags().contains_key("natural")
                    || obj.tags().contains_key("landuse")
                )
            )
        )
}


/// Nametiles generator
///
/// Generates a flatgeobuffers file which can be then converted to nametiles using tippecanoe
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path of the .osm.pbf file. Get one at https://www.geofabrik.de/data/download.html
    input: PathBuf,
}

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let args = Args::parse();

    log::info!("Started");
    let mut pbf_reader = OsmPbfReader::new(File::open(args.input).unwrap());
    log::info!("OsmPbfReader created");

    log::info!("Finding all boundary relations and their members...");

    struct RelMember {
        way_id: i64,
        rel_id: i64
    }
    //let mut last_logged = Instant::now();
    //
    //
    let mut tmp_db = KvStoreOSM(kv::Store::new(kv::Config::new("/nix/temporary/nametilesgen/db")).unwrap());


    if true {
        pbf_reader
            .get_objs_and_deps_store(include_in_tiles, &mut tmp_db)
            .unwrap();
    }


    log::info!("...done");

    //log::info!("");

    let objs_of_interest_bucket = tmp_db.osm_elems_bucket();

    //log::info!("Total of {} elements were extracted from the PBF file.", objs_of_interest_bucket.len());
    log::info!("Constructing polygons from the relations");

    let relations = objs_of_interest_bucket.iter()
        .map(|it| it.unwrap().value().unwrap())
        .filter_map(|obj: kv::Bincode<osmpbfreader::OsmObj>| obj.0.relation().cloned());

    struct ProcessedBoundary {
        pub boundary: LineString,
        pub name: String
    }

    let relation_processing_results = relations.par_bridge().map(|relation| {
        if !include_in_tiles(&relation.clone().into()) {
            return Result::Err(());
        }
        let Some(relation_name) = get_name(&relation.tags) else {
            return Result::Err(());
        };

        log::trace!("Processing relation {}", relation.id.0);

        let opt_unused_member_ways = relation.refs.iter()
            .filter(|obj|
                obj.member.is_way()
                    && obj.role == "outer" // For now, we ignore enclaves and inner ways
            )
            .map(|way| objs_of_interest_bucket.get(&osm_id_to_kv_key(&way.member)).unwrap().map(|w| w.0.way().unwrap().to_owned()))
            .collect::<Option<Vec<_>>>();
        let Some(mut unused_member_ways) = opt_unused_member_ways else {
            log::debug!("Relation {} ({}) is incomplete, skipping it",
                relation.id.0, relation_name // We filtered the relations to all contain names
            );
            return Result::Err(());//continue 'relation_loop;
        };

        //let way_info = HashMap::new();
        //
        let mut polygons = Vec::new();

        while !unused_member_ways.is_empty() { // This loop iterates over all outer rings of the polygon
            let Some(first_way) = unused_member_ways.pop() else {
                log::debug!("Relation {} ({}) contains no ways, skipping it",
                    relation.id.0, relation_name // We filtered the relations to all contain names
                );
                return Result::Err(());//continue 'relation_loop;
            };

            let Some((mut first_node, mut current_node)) = first_and_last_node(&objs_of_interest_bucket, &first_way) else {
                log::debug!("Relation {} ({}) contains an incomplete way ({}), skipping it",
                    relation.id.0,
                    relation_name, // We filtered the relations to all contain names
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
                            relation_name, // We filtered the relations to all contain names
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
                        relation_name,
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
            polygons.push(geo::LineString::new(polygon_nodes));
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
        Result::Ok(polygons.into_iter().map(move |boundary| ProcessedBoundary {
            boundary,
            name: relation_name.clone()
        }).par_bridge())
    });


    let (polygons_tx, polygons_rx) = mpsc::channel::<ProcessedBoundary>();

    let mut fgb_writer = FgbWriter::create("nametiles_base", flatgeobuf::GeometryType::MultiPolygon).unwrap();
    fgb_writer.dataset_begin(None).unwrap();

    log::info!("Initialized the FGB dataset");

    thread::spawn(move || {
        for (idx, ProcessedBoundary { boundary, name }) in polygons_rx.iter().enumerate() {
            fgb_writer.feature_begin(idx as u64).unwrap();
            fgb_writer.properties_begin().unwrap();
            fgb_writer.property(0, "name", &geozero::ColumnValue::String(&name)).unwrap();
            //fgb_writer.property();
            fgb_writer.properties_end().unwrap();
            fgb_writer.geometry_begin().unwrap();
            fgb_writer.polygon_begin(true, 1, 0).unwrap();
            fgb_writer.linestring_begin(false, boundary.0.len(), 0).unwrap();
            for (idx, coord) in boundary.0.iter().enumerate() {
                fgb_writer.xy(coord.x, coord.y, idx).unwrap();
            }
            fgb_writer.linestring_end(false, 0).unwrap();
            fgb_writer.polygon_end(true, 0).unwrap();

            fgb_writer.geometry_end().unwrap();

            fgb_writer.feature_end(idx as u64).unwrap();
        }
        fgb_writer.dataset_end().unwrap();

        fgb_writer.write(File::create("dataset.fgb").unwrap()).unwrap();
    });

    relation_processing_results
        .filter_map(|res| res.ok()) // TODO: Don't ignore failures
        .flatten()
        .for_each(|boundary| {
            polygons_tx.send(boundary).unwrap();
                //println!("Sending..");
        });
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
    let nodes = found_way.nodes.into_iter().map(|node|
        {
            let bc = objs_of_interest_bucket
                .get(&osm_id_to_kv_key(&OsmId::Node(node)))
                .unwrap()
                .unwrap();
            let node = bc.0.node().unwrap();
            Coord {
                x: node.lon(),
                y: node.lat(),
            }
        }
    );
    nodes
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
