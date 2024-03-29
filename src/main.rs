#![feature(ptr_internals)]
#![feature(allocator_api)]

use std::time::SystemTime;

use palette::{Hsl, Srgb};
use plotters::{
    drawing::IntoDrawingArea,
    prelude::{
        BitMapBackend, ChartBuilder, Color, IntoFont, LineSeries, RGBColor, BLACK, RED, WHITE,
    },
};
use statrs::statistics::Statistics;

use crate::{
    a_star::Path,
    bidirectional::bi_astar::a_star_bi,
    bounds::{Boundable, Bounds},
    osm_parser::OpenStreetMap,
    params::SimpleParams,
};

mod a_star;
mod bidirectional;
mod bounds;
mod compact_array;
mod osm_parser;
mod params;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let map = OpenStreetMap::parse("minnesota-latest.osm.pbf")?;
    // let map = map.trim(); // to prevent unsolvable paths
    // map.save("map.save")?;
    let map = OpenStreetMap::read_custom_file("map.save")?;

    let repeat = 1000;

    let start = SystemTime::now();

    let mut paths = Vec::with_capacity(repeat);

    let mut time = SystemTime::now();

    for i in 0..repeat {
        let (init_id, init) = map.random();
        let (goal_id, goal) = map.random();

        if i % 100 == 0 {
            println!("path {}", i);
        }

        let path = a_star_bi(&map, init_id, goal_id, &SimpleParams)
            .unwrap_or_else(|| panic!("no path found between {:?} and {:?}", init, goal));

        paths.push((path, time.elapsed().unwrap()));
        time = SystemTime::now();
    }

    // DEBUG: 41982ms for 1000 bi-directional, 193386ms for 1000 regular ...
    // bidirectional is ~4.6x faster RELEASE: 3415ms and 20114ms respectively
    // ~5.8x speedup
    println!("total time {} ms", start.elapsed().unwrap().as_millis());

    let paths: Vec<_> = paths.into_iter().map(|(path, _)| path).collect();

    let mean_miles = paths.iter().map(|path| path.length_miles()).mean();
    let mean_nodes = paths.iter().map(|path| path.ids.len() as f64).mean();

    println!(
        "mean miles {:.2}mi, mean nodes {:.2} ",
        mean_miles, mean_nodes
    );

    draw(&map, &paths)?;

    Ok(())
}

fn draw(map: &OpenStreetMap, paths: &[Path]) -> Result<(), Box<dyn std::error::Error>> {
    let Bounds { from, to } = map.get_bounds();

    let root = BitMapBackend::new("5.png", (1000, 2000)).into_drawing_area();
    root.fill(&WHITE)?;
    let root = root.margin(10, 10, 10, 10);

    let mut chart = ChartBuilder::on(&root)
        // Set the caption of the chart
        .caption("Paths", ("sans-serif", 40).into_font())
        // Set the size of the label region
        .x_label_area_size(20)
        .y_label_area_size(40)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_cartesian_2d(from.x()..to.x(), from.y()..to.y())?;

    println!("built chart");

    // Then we can draw a mesh
    chart.configure_mesh().x_labels(5).y_labels(5).draw()?;

    print!("built mesh");

    println!("drawing...");

    let repeat = paths.len();
    for (i, path) in paths.iter().enumerate() {
        let path_points: Vec<_> = path
            .ids
            .iter()
            .map(|&id| map.get(id).location.f64())
            .collect();

        let prop = (i as f64) / (repeat as f64);

        let hsl = Hsl::new(prop * 360.0, 1.0, 0.4);
        let rgb = Srgb::from(hsl);

        let red = (rgb.red * 255.0) as u8;
        let green = (rgb.green * 255.0) as u8;
        let blue = (rgb.blue * 255.0) as u8;

        let rgb_color = RGBColor(red, green, blue);
        let rgba_color = rgb_color.mix(0.4);

        // let rgb_color = RGBColor(red, green, blue);
        chart.draw_series(LineSeries::new(path_points, &rgba_color))?;
    }
    Ok(())
}
