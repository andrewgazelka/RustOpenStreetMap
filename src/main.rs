#![feature(ptr_internals)]
#![feature(allocator_api)]

use std::time::SystemTime;

use palette::{Hsl, Srgb};
use plotters::drawing::IntoDrawingArea;
use plotters::prelude::{BitMapBackend, BLACK, ChartBuilder, IntoFont, LineSeries, RED, RGBColor, WHITE};

use crate::bidirectional::bi_astar::a_star_bi;
use crate::bounds::{Boundable, Bounds};
use crate::osm_parser::OpenStreetMap;
use crate::utils::parse_pre_generated;

mod osm_parser;
mod a_star;
mod compact_array;
mod bounds;
mod utils;
mod bidirectional;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let map = parse_pre_generated()?;

    let repeat = 10;

    let mut paths = Vec::with_capacity(repeat);

    let mut time = SystemTime::now();

    for i in 0..repeat {
        let (init_id, init) = map.random();
        let (goal_id, goal) = map.random();

        println!("path {} init {}, goal {}", i, init_id, goal_id);

        let path = a_star_bi(&map, init_id, goal_id)
            .unwrap_or_else(|| panic!("no path found between {:?} and {:?}", init, goal));

        paths.push((path, time.elapsed().unwrap()));
        time = SystemTime::now();
    }

    println!("miles, ns, init, goal");
    for (path, time) in paths {
        println!("{:.2}, {} {} {}", path.ids.len(), time.as_nanos(), path.ids[0], path.ids.last().unwrap());
    }


    Ok(())
}
