use std::io::Error;
use crate::osm_parser::Map;

mod osm_parser;
mod a_star;

fn main() -> Result<(), Error> {
    let map = Map::parse()?;
    if let Some(node) = map.closest(45.198653799999995, -92.692009) {
        println!("mhmmm {:?}", node);
        println!("dist {}", node.dist_miles())
    }
    Ok(())
}
