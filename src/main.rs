#![feature(ptr_internals)]
#![feature(allocator_api)]

use std::io;
use std::io::BufRead;

use crate::osm_parser::OpenStreetMap;

mod osm_parser;
mod a_star;
mod compact_array;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let map = OpenStreetMap::parse("minnesota-latest.osm.pbf")?;
    println!("returned with {} nodes", map.node_count());

    let stdin = io::stdin();

    loop {
        let coords: Vec<f64> = stdin.lock().lines().take(4).map(|item| item.unwrap().trim().parse().unwrap()).collect();

        // current loc
        let init = map.closest(coords[0], coords[1]).expect("no closest result for init");


        // minneapolis
        let goal = map.closest(coords[2], coords[3]).expect("guthrie");

        println!("{:?} -> {:?}", map.get(init.id).location, map.get(goal.id).location);
        //
        println!("finding path");
        if let Some(path) = a_star::path(&map, init.id, goal.id) {
            println!("found path of length {}", path.len());
            let locations = path.into_iter().map(|id| map.get(id)).map(|node| node.location);
            let mut prev_loc = None;
            let mut total = 0.0;
            for loc in locations {
                let dx = match prev_loc {
                    Some(prev) => loc.dist2(prev).sqrt() as f64,
                    None => 0.0
                };
                prev_loc = Some(loc);
                total += dx;
            }
            println!("total miles {}", total * 68.703);
        } else {
            println!("no path");
        }
    }
}
