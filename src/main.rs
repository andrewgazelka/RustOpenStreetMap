#![feature(ptr_internals)]
#![feature(allocator_api)]


use crate::osm_parser::OpenStreetMap;

mod osm_parser;
mod a_star;
mod compact_array;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let map = OpenStreetMap::parse("minnesota-latest.osm.pbf")?;
    println!("returned with {} nodes", map.node_count());

    // current loc
    let init = map.closest(46.5296, -93.7059).expect("no closest result for init");


    // jjjjjjj
    // // minneapolis
    // let goal = map.closest(44.9778, -93.2650).expect("no closest result for goal");
    let goal = map.closest(43.9834, -94.6254).expect("st james oof");
    // 
    println!("finding path");
    let path = a_star::path(&map, init.id, goal.id).expect("no path found");
    // 
    println!("found path of length {}", path.len());

    // 
    // let root = BitMapBackend::new("4.png", (640, 480)).into_drawing_area();
    // 
    // root.fill(&RGBColor(240, 200, 200))?;
    // 
    // let locations: Vec<Location> = path.iter().map(|id| map.get(*id)).map(|node| node.location).collect();
    // 
    // 
    // let minX = f64::from(locations.iter().map(|Location(x, y)| NotNan::new(*x).unwrap()).min().unwrap());
    // let minY = f64::from(locations.iter().map(|Location(x, y)| NotNan::new(*y).unwrap()).min().unwrap());
    // let maxX = f64::from(locations.iter().map(|Location(x, y)| NotNan::new(*x).unwrap()).max().unwrap());
    // let maxY = f64::from(locations.iter().map(|Location(x, y)| NotNan::new(*y).unwrap()).max().unwrap());
    // 
    // 
    // let mut chart = ChartBuilder::on(&root)
    //     .build_cartesian_2d(minX..maxX, minY..maxY)?;
    // 
    // let x: Vec<_> = locations.into_iter().map(|Location(a, b)| (a, b)).collect();
    // chart.draw_series(LineSeries::new(x, &RED))?;
    // 
    // println!("f");
    Ok(())
}
