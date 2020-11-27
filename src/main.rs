use ordered_float::NotNan;
use plotters::drawing::IntoDrawingArea;
use plotters::element::{Circle, Text};
use plotters::prelude::{BitMapBackend, BLACK, Cartesian2d, EmptyElement, IntoFont, RGBColor, ShapeStyle};

use crate::osm_parser::{Location, OpenStreetMap};
use plotters::coord::types::RangedCoordf64;

mod osm_parser;
mod a_star;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let map = OpenStreetMap::parse("map.osm.pbf")?;
    println!("parsed with {} nodes", map.node_count());

    // current loc
    let init = map.closest(45.198653799999995, -92.692009).expect("no closest result for init");

    // minneapolis
    let goal = map.closest(44.9778, -93.2650).expect("no closest result for goal");

    println!("finding path");
    let path = a_star::path(&map, init.node, goal.node).expect("no path found");

    println!("found path of length {}", path.len());

    let root = BitMapBackend::new("4.png", (640, 480)).into_drawing_area();

    root.fill(&RGBColor(240, 200, 200))?;


    let dot_and_label = |x: f64, y: f64| {
        return EmptyElement::at((x, y))
            + Circle::new((0, 0), 3, ShapeStyle::from(&BLACK).filled())
            + Text::new(
            format!("({:.2},{:.2})", x, y),
            (10, 0),
            ("sans-serif", 15.0).into_font(),
        );
    };

    let locations: Vec<Location> = path.iter().map(|id| map.node_map[id]).collect();


    let minX = f64::from(locations.iter().map(|Location(x, y)| NotNan::new(*x).unwrap()).min().unwrap());
    let minY = f64::from(locations.iter().map(|Location(x, y)| NotNan::new(*y).unwrap()).min().unwrap());
    let maxX = f64::from(locations.iter().map(|Location(x, y)| NotNan::new(*x).unwrap()).max().unwrap());
    let maxY = f64::from(locations.iter().map(|Location(x, y)| NotNan::new(*y).unwrap()).max().unwrap());

    let root = root.apply_coord_spec(Cartesian2d::<RangedCoordf64, RangedCoordf64>::new(
        minX..maxX,
        minY..maxY,
        (0..640, 0..480),
    ));

    locations.iter().for_each(|Location(x, y)| {
        println!("draw {} {} ", x, y);
        root.draw(&dot_and_label(*x, *y));
    });

    println!("f");
    Ok(())
}
