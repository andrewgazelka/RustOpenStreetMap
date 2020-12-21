use std::io;

use crate::osm_parser::OpenStreetMap;
use crate::a_star::Path;
use plotters::prelude::{BitMapBackend, BLACK, ChartBuilder, RGBColor, LineSeries};
use plotters::drawing::IntoDrawingArea;
use crate::bounds::{Bounds, Boundable};
use palette::{Hsl, Srgb};

pub type Res<T> = Result<T, io::Error>;
pub type DynRes<T> = Result<T, Box<dyn std::error::Error>>;
pub type EmptyRes = Res<()>;

const EmptyOk: Res<()> = Ok(());

fn save() -> EmptyRes {
    let all_map = OpenStreetMap::parse("minnesota-latest.osm.pbf")?;
    let map = all_map.trim();
    map.save("map.save")?;
    EmptyOk
}

pub fn parse_pre_generated() -> Res<OpenStreetMap> {
    OpenStreetMap::read_custom_file("map.save")
}

pub fn generate_images(paths: &Vec<Path>) -> DynRes<()>{
    let first = &paths[0];
    let map = first.parent_map;
    let root = BitMapBackend::new("5.png", (10000, 20000)).into_drawing_area();
    root.fill(&BLACK)?;
    let root = root.margin(10, 10, 10, 10);
    // After this point, we should be able to draw construct a chart context

    let Bounds{from, to} = map.get_bounds();

    println!("found span {:?} -> {:?}", from, to);

    let mut chart = ChartBuilder::on(&root)
        .build_cartesian_2d(from.x()..to.x(), from.y()..to.y())?;
    
    chart
        .configure_mesh()
        .draw()?;
    
    let size = paths.len() as f64;

    for (i,path) in paths.iter().enumerate() {
        println!("path {}", i);

        let path_points: Vec<_> = path.ids.iter().map(|&id| map.get(id).location.f64()).collect();

        let prop = (i as f64) / size;

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
