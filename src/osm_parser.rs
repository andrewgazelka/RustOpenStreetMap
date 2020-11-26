use std::cmp::Ordering;
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

pub struct Map {
    pub connections: HashMap<i64, Vec<i64>>,
    pub node_map: HashMap<i64, Node>,
}

#[derive(Debug, Copy, Clone)]
pub struct Node(f64, f64);

#[derive(Debug)]
pub struct ClosestResult {
    id: i64,
    dist: f64,
    node: Node,
}

impl ClosestResult {
    pub fn dist_miles(&self) -> f64 {
        self.dist *68.703
    }
}

impl Map {
    pub(crate) fn closest(&self, lat: f64, long: f64) -> Option<ClosestResult> {
        let mut min_id = None;
        let mut min_val = f64::MAX;
        self.node_map.iter().for_each(|(id, Node(nlat, nlong))| {
            let dlat = nlat - lat;
            let dlong = nlong - long;
            let num = dlat * dlat + dlong * dlong;
            if num < min_val {
                min_val = num;
                min_id = Some(*id)
            }
        });
        min_id.map(|id| ClosestResult {
            id,
            dist: min_val,
            node: self.node_map.get(&id).unwrap().clone(),
        })
    }

    pub(crate) fn parse() -> Result<Map, io::Error> {
        let reader = ElementReader::from_path("minnesota-latest.osm.pbf")?;
        let mut connections = HashMap::new();
        let mut node_map = HashMap::new();

        reader.for_each(|element| {
            match element {
                Element::Way(way) => {
                    process_way(&mut connections, way)
                }
                Element::Node(node) => {
                    let to_insert = Node(node.lat(), node.lon());
                    node_map.insert(node.id(), to_insert);
                }

                Element::DenseNode(node) => {
                    let to_insert = Node(node.lat(), node.lon());
                    node_map.insert(node.id, to_insert);
                }
                Element::Relation(_) => {} // we do not care about relations
            }
        })?;


        println!("done");
        Ok(Map {
            connections,
            node_map,
        })
    }
}


#[cfg(test)]
mod tests {
    use std::io;

    use crate::osm_parser::Map;

    #[test]
    fn exploration() -> Result<(), io::Error> {
        let map = Map::parse()?;
        let node = map.closest(45.198653799999995, -92.692009);
        println!("mhmmm {:?}", node);
        Ok(())
    }
}
