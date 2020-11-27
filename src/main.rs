use ordered_float::NotNan;
use plotters::coord::types::RangedCoordf64;
use plotters::drawing::IntoDrawingArea;
use plotters::element::{Circle, Text};
use plotters::prelude::{BitMapBackend, BLACK, Cartesian2d, ChartBuilder, EmptyElement, IntoFont, LineSeries, RED, RGBColor, ShapeStyle};

use crate::osm_parser::{Location, OpenStreetMap};

mod osm_parser;
mod a_star;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let map = OpenStreetMap::parse("minnesota-latest.osm.pbf")?;
    println!("parsed with {} nodes", map.node_count());

    return Ok(());
    // current loc
    let init = map.closest(45.198653799999995, -92.692009).expect("no closest result for init");

    // minneapolis
    let goal = map.closest(44.9778, -93.2650).expect("no closest result for goal");

    println!("finding path");
    let path = a_star::path(&map, init.node, goal.node).expect("no path found");

    println!("found path of length {}", path.len());

    let root = BitMapBackend::new("4.png", (640, 480)).into_drawing_area();

    root.fill(&RGBColor(240, 200, 200))?;

    let locations: Vec<Location> = path.iter().map(|id| map.node_map[id]).collect();


    let minX = f64::from(locations.iter().map(|Location(x, y)| NotNan::new(*x).unwrap()).min().unwrap());
    let minY = f64::from(locations.iter().map(|Location(x, y)| NotNan::new(*y).unwrap()).min().unwrap());
    let maxX = f64::from(locations.iter().map(|Location(x, y)| NotNan::new(*x).unwrap()).max().unwrap());
    let maxY = f64::from(locations.iter().map(|Location(x, y)| NotNan::new(*y).unwrap()).max().unwrap());


    let mut chart = ChartBuilder::on(&root)
        .build_cartesian_2d(minX..maxX, minY..maxY)?;

    let x: Vec<_> = locations.into_iter().map(|Location(a, b)| (a, b)).collect();
    chart.draw_series(LineSeries::new(x, &RED))?;

    println!("f");
    Ok(())
}
