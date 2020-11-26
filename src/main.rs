use std::io::Error;

use crate::osm_parser::test;

mod osm_parser;
mod a_star;

fn main() -> Result<(), Error> {
    let x = test()?;
    x.map.iter().take(20).for_each(|(id, adj)| {
        if let Some(node) = x.node_map.get(id) {
            print!("{:?}", node)
        }
        println!(" ::: {:?}", adj)
    });
    println!("Done");
    Ok(())
}
