use std::collections::HashMap;
use std::io;

use osmpbf::ElementReader;

use crate::compact_array::{CompactVec, CompactVecIterator};

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

fn process_way(id_to_idx: &mut HashMap<i64, u32>, idx_to_node: &mut Vec<Node>, way: &osmpbf::Way) {
    let refs: Vec<_> = way.refs().map(|real_id| {
        *id_to_idx.get(&real_id).unwrap()
    }).collect();

    if refs.len() <= 1 {
        return;
    }

    let first_idx = *refs.first().unwrap();

    let first_node = idx_to_node.get_mut(first_idx as usize).unwrap();


    first_node.connected.push(refs[1]);

    for i in 1..(refs.len() - 1) {
        let prev_idx = refs[i - 1];
        let on_idx = refs[i];
        let next_idx = refs[i + 1];
        let node = idx_to_node.get_mut(on_idx as usize).unwrap();
        node.connected.push2(prev_idx, next_idx);
    }

    let last_idx = *refs.last().unwrap();

    let last_node = idx_to_node.get_mut(last_idx as usize).unwrap();
    last_node.connected.push(refs[refs.len() - 2])


    // for i in 1..(refs.len()-1) {
    //     let prev_idx = refs[i-1];
    //     let prev_node = idx_to_node.get_mut(prev_idx as usize).unwrap();
    //
    //     let next_idx = refs[i];
    // }

    // let ref_len = (refs.len() - 1) as u8; // - 1 because no ref to self
    //
    // assert!(ref_len > 0); // if this is not true why is this a way
    //
    // refs.iter().for_each(|id_a| {
    //
    //     let compact_vec = &mut node_a.connected;
    //
    //     let mut i_on = compact_vec.len();
    //     compact_vec.add_len(ref_len);
    //
    //     refs.iter().for_each(|id_b| {
    //         compact_vec.insert(i_on, *id_b);
    //         i_on += 1;
    //     });
    // });
}

pub struct OpenStreetMap {
    idx_to_node: Vec<Node>,
}


#[derive(Debug, Copy, Clone)]
pub struct Location(pub f64, pub f64);

impl Location {
    pub fn dist2(&self, other: Location) -> f64 {
        let dx = self.0 - other.0;
        let dy = self.1 - other.1;
        dx * dx + dy * dy
    }
}

#[repr(packed)]
pub struct Node {
    pub connected: CompactVec<u32>,
    pub location: Location,
}

impl Node {
    pub fn dist2(&self, other: &Node) -> f64 {
        let loc = self.location;
        loc.dist2(other.location)
    }
}

#[derive(Debug)]
pub struct ClosestResult {
    pub dist: f64,
    pub id: u32,
}

impl ClosestResult {
    #[allow(dead_code)]
    pub fn dist_miles(&self) -> f64 {
        self.dist * 68.703
    }
}


impl OpenStreetMap {
    pub fn get(&self, id: u32) -> &Node {
        self.idx_to_node.get(id as usize).unwrap()
    }
    pub fn node_count(&self) -> usize {
        self.idx_to_node.len()
    }

    pub fn next_to_id(&self, from_id: u32) -> CompactVecIterator<'_, u32> {
        self.get(from_id).connected.iterator()
    }

    #[allow(dead_code)]
    pub fn closest(&self, lat: f64, long: f64) -> Option<ClosestResult> {
        let mut min_id = None;
        let mut min_val = f64::MAX;
        self.idx_to_node.iter().enumerate().for_each(|(id, node)| {
            let Location(nlat, nlong) = node.location;
            let dlat = nlat - lat;
            let dlong = nlong - long;
            let num = dlat * dlat + dlong * dlong;
            if num < min_val {
                min_val = num;
                min_id = Some(id)
            }
        });
        min_id.map(|_| ClosestResult {
            id: min_id.unwrap() as u32,
            dist: min_val,
        })
    }

    pub(crate) fn parse(name: &str) -> Result<OpenStreetMap, io::Error> {
        let mut id_to_idx = HashMap::new();
        let mut idx_to_node = Vec::new();

        {
            let reader1 = ElementReader::from_path(name)?;
            reader1.for_each(|element| {
                match element {
                    osmpbf::Element::Way(way) => {
                        process_way(&mut id_to_idx, &mut idx_to_node, &way)
                    }
                    osmpbf::Element::Node(node) => {
                        id_to_idx.insert(node.id(), idx_to_node.len() as u32);
                        let location = Location(node.lat(), node.lon());
                        let to_insert = Node {
                            location,
                            connected: CompactVec::empty(),
                        };
                        idx_to_node.push(to_insert);
                    }

                    osmpbf::Element::DenseNode(node) => {
                        id_to_idx.insert(node.id, idx_to_node.len() as u32);
                        let location = Location(node.lat(), node.lon());
                        let to_insert = Node {
                            location,
                            connected: CompactVec::empty(),
                        };
                        idx_to_node.push(to_insert);
                    }
                    _ => {} // we do not care about relations
                }
            })?;
        }

        println!("returning");

        Ok(OpenStreetMap {
            idx_to_node,
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
