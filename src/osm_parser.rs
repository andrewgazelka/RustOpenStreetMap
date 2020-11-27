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
        let vec = match map.get_mut(id) {
            Some(v) => v,
            None => {
                let new_vec = Vec::new();
                map.insert(*id, new_vec); // TODO: better way?
                map.get_mut(id).unwrap()
            }
        };

        refs.iter().for_each(|x|
            if id != x {
                vec.push(*x)
            }
        );

    });
}

pub struct OpenStreetMap {
    pub connections: HashMap<i64, Vec<i64>>,
    pub node_map: HashMap<i64, Location>,
}


#[derive(Debug, Copy, Clone)]
pub struct Location(pub f64, pub f64);

impl Location {
    pub fn dist2(&self, other: &Location) -> f64 {
        let dx = self.0 - other.0;
        let dy = self.1 - other.1;
        dx * dx + dy * dy
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Node {
    pub id: i64,
    pub location: Location,
}

impl Node {
    pub fn dist2(&self, other: &Node) -> f64 {
        self.location.dist2(&other.location)
    }
}

#[derive(Debug)]
pub struct ClosestResult {
    pub dist: f64,
    pub node: Node,
}

impl ClosestResult {
    pub fn dist_miles(&self) -> f64 {
        self.dist * 68.703
    }
}

impl OpenStreetMap {
    pub fn node_count(&self) -> usize {
        self.node_map.len()
    }

    pub fn next_to_id(&self, from_id: i64) -> impl Iterator<Item=Node> + '_ {
        let conns = self.connections.get(&from_id).unwrap();
        conns.iter().map(move |id| Node { // TODO what is move
            location: *self.node_map.get(id).unwrap(),
            id: *id,
        })
    }

    pub fn next_to(&self, node: Node) -> impl Iterator<Item=Node> + '_ {
        let conns = self.connections.get(&node.id).unwrap();
        conns.iter().map(move |id| Node { // TODO what is move
            location: *self.node_map.get(id).unwrap(),
            id: *id,
        })
    }

    pub fn closest(&self, lat: f64, long: f64) -> Option<ClosestResult> {
        let mut min_id = None;
        let mut min_val = f64::MAX;
        self.node_map.iter().for_each(|(id, Location(nlat, nlong))| {
            let dlat = nlat - lat;
            let dlong = nlong - long;
            let num = dlat * dlat + dlong * dlong;
            if num < min_val {
                min_val = num;
                min_id = Some(*id)
            }
        });
        min_id.map(|id| ClosestResult {
            node: Node {
                id,
                location: *self.node_map.get(&id).unwrap(),
            },
            dist: min_val,
        })
    }

    pub(crate) fn parse(name: &str) -> Result<OpenStreetMap, io::Error> {
        let reader = ElementReader::from_path(name)?;
        let mut connections = HashMap::new();
        let mut node_map = HashMap::new();

        reader.for_each(|element| {
            match element {
                Element::Way(way) => {
                    process_way(&mut connections, way)
                }
                Element::Node(node) => {
                    let to_insert = Location(node.lat(), node.lon());
                    node_map.insert(node.id(), to_insert);
                }

                Element::DenseNode(node) => {
                    let to_insert = Location(node.lat(), node.lon());
                    node_map.insert(node.id, to_insert);
                }
                Element::Relation(_) => {} // we do not care about relations
            }
        })?;

        println!("done");
        Ok(OpenStreetMap {
            connections,
            node_map,
        })
    }
}


// #[cfg(test)]
// mod tests {
//     use std::io;
//
//     use crate::osm_parser::OpenStreetMap;
//
//     #[test]
//     fn exploration() -> Result<(), io::Error> {
//         let map = OpenStreetMap::parse()?;
//         let node = map.closest(45.198653799999995, -92.692009);
//         println!("mhmmm {:?}", node);
//         Ok(())
//     }
// }
