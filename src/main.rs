#![feature(ptr_internals)]
#![feature(allocator_api)]

use std::time::SystemTime;

use palette::{Hsl, Srgb};
use plotters::drawing::IntoDrawingArea;
use plotters::prelude::{BitMapBackend, ChartBuilder, IntoFont, LineSeries, RED, WHITE, RGBColor, BLACK};

use crate::osm_parser::{OpenStreetMap};
use crate::bounds::{Boundable, Bounds};

mod osm_parser;
mod a_star;
mod compact_array;
mod bounds;

fn parse() -> Result<(), Box<dyn std::error::Error>> {
    println!("starting parse");
    let all_map = OpenStreetMap::parse("minnesota-latest.osm.pbf")?;
    println!("trimming");
    let map = all_map.trim();
    println!("saving");
    map.save("map.save")?;
    println!("saved");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("reading file");
    let map = OpenStreetMap::read_custom_file("map.save")?;
    println!("done reading file");

    let time = SystemTime::now();

    let repeat = 100;

    let mut paths = Vec::with_capacity(repeat);
    for i in 0..repeat {
        println!("path {}", i);
        let (init_id, init) = map.random();
        let (goal_id, goal) = map.random();

        let path = a_star::path(&map, init_id, goal_id).unwrap_or_else(|| panic!("no path found between {:?} and {:?}", init, goal));
        paths.push(path);
    }
    let total_time = time.elapsed().unwrap().as_millis();

    println!("total time {} ms", total_time);
    println!("avg time {} ms", total_time as f64 / repeat as f64);

    let total_miles: f64 = paths.iter().map(|path| map.length_miles(path)).sum();
    println!("avg miles {}", total_miles / repeat as f64);

    let root = BitMapBackend::new("5.png", (10000, 20000)).into_drawing_area();
    root.fill(&BLACK)?;
    let root = root.margin(10, 10, 10, 10);
    // After this point, we should be able to draw construct a chart context

    let Bounds{from, to} = map.get_bounds();

    println!("found span {:?} -> {:?}", from, to);

    let mut chart = ChartBuilder::on(&root)
        // Set the caption of the chart
        .caption("Paths", ("sans-serif", 40).into_font())
        // Set the size of the label region
        // .x_label_area_size(20)
        // .y_label_area_size(40)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_cartesian_2d(from.x()..to.x(), from.y()..to.y())?;

    println!("built chart");

    // Then we can draw a mesh
    chart
        .configure_mesh()
        // .x_labels(5)
        // .y_labels(5)
        // .y_label_formatter(&|x| format!("{:.3}", x))
        .draw()?;

    print!("built mesh");


    println!("drawing...");

    for i in 0..repeat {
        println!("path {}", i);

        let path_points: Vec<_> = paths[i].ids.iter().map(|&id| map.get(id).location.f64()).collect();

        let prop = (i as f64) / repeat as f64;

        let hsl = Hsl::new(360.0*prop, 1.0, 0.4);

        let rgb = Srgb::from(hsl);

        let red = (rgb.red * 255.0) as u8;
        let green = (rgb.green * 255.0) as u8;
        let blue = (rgb.blue * 255.0) as u8;

        println!("rgb {} {} {}", red, green, blue);
        let rgb_color = RGBColor(red, green, blue);
        println!("draw len {}", path_points.len());
        chart.draw_series(LineSeries::new(
            path_points,
            &rgb_color,
        ))?;
    }

    Ok(())
}
