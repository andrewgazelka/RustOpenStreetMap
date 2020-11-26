use std::collections::HashMap;
use std::io;

use osmpbf::{Element, ElementReader, Way};

/**

Nodes, ways, etc.
https://labs.mapbox.com/mapping/osm-data-model/#:~:text=Attributes%20are%20described%20as%20tags,that%20represent%20a%20larger%20whole.

OSM XML
https://wiki.openstreetmap.org/wiki/OSM_XML

File Formats
https://osmcode.org/file-formats-manual/
    PBF

https://download.geofabrik.de/north-america/us/minnesota.html

https://wiki.openstreetmap.org/wiki/PBF_Format#Encoding_OSM_entities_into_fileblocks

<osm>
  <node id lat lon>
  <way
    <nd ref>
*/

fn process_way(map: &mut HashMap<i64, Vec<i64>>, way: Way) {
    let refs: Vec<i64> = way.refs().collect();
    refs.iter().for_each(|id| {
        if let Some(vec) = map.get_mut(id) {
            refs.iter().for_each(|x|
                vec.push(*x)
            );
        } else {
            // TODO: huh
            let x = vec![*id];
            map.insert(*id, x);
        }
    });
}

pub struct Nodes {
    pub map: HashMap<i64, Vec<i64>>,
    pub node_map: HashMap<i64, Node>,
}

#[derive(Debug)]
pub struct Node(f64, f64);


pub fn test() -> Result<Nodes, io::Error> {
    let reader = ElementReader::from_path("minnesota-latest.osm.pbf")?;
    let mut map = HashMap::new();
    let mut node_map = HashMap::new();

    reader.for_each(|element| {
        match element {
            Element::Way(way) => {
                process_way(&mut map, way)
            }
            Element::Node(node) => {
                let to_insert = Node(node.lon(), node.lat());
                node_map.insert(node.id(), to_insert);
            }

            Element::DenseNode(node) => {
                let to_insert = Node(node.lon(), node.lat());
                node_map.insert(node.id, to_insert);
            }
            Element::Relation(_) => {} // we do not care about relations
        }
    })?;


    println!("done");
    Ok(Nodes {
        map,
        node_map,
    })
}
