use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::ptr::read;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use osmpbf::ElementReader;
use rand::Rng;

use crate::a_star::Path;
use crate::compact_array::{CompactVec, CompactVecIterator};
use std::slice::Iter;

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
#[repr(packed)]
#[derive(Debug)]
pub struct Node {
    pub connected: CompactVec<u32>,
    pub location: Location,
}

fn process_way(id_to_idx: &mut HashMap<i64, u32>, idx_to_node: &mut Vec<Node>, way: &osmpbf::Way) {
    let valid = OpenStreetMap::valid_way(way);

    if !valid {
        return;
    }

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
}

pub struct OpenStreetMap {
    idx_to_node: Vec<Node>,
}

#[derive(Debug, Copy, Clone)]
#[repr(packed)]
pub struct Location(pub f64, pub f64);

impl Location {
    pub fn dist2(&self, other: Location) -> f64 {
        let dx = self.0 - other.0;
        let dy = self.1 - other.1;
        dx * dx + dy * dy
    }

    pub fn f32(&self) -> (f32, f32) {
        (self.0 as f32, self.1 as f32)
    }

    pub fn f64(&self) -> (f64, f64) {
        (self.0, self.1)
    }
}

impl Node {
    #[allow(dead_code)]
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

impl Location {
    pub fn x(&self) -> f64 {
        self.0
    }

    pub fn y(&self) -> f64 {
        self.1
    }
}

impl OpenStreetMap {

    pub fn save(&self, name: &str) -> Result<(), io::Error> {
        let file = File::create(name)?;
        let mut writer = BufWriter::new(file);

        writer.write_u32::<BigEndian>(self.idx_to_node.len() as u32)?;


        for node in &self.idx_to_node {
            let Location(x, y) = node.location; // 8*2 bytes
            writer.write_f64::<BigEndian>(x)?;
            writer.write_f64::<BigEndian>(y)?;
            let connected_len = node.connected.len(); // 4*connected_len bytes + 1 byte
            writer.write_u8(connected_len)?;
            for &i in node.connected.iterator() {
                writer.write_u32::<BigEndian>(i)?;
            }
        }
        writer.flush()?;

        Ok(())
    }
    
    pub fn iterator(&self) -> Iter<'_, Node> {
        self.idx_to_node.iter()
    }

    pub fn read_custom_file(name: &str) -> Result<OpenStreetMap, io::Error> {
        let file = File::open(name)?;
        let mut reader = BufReader::new(file);
        let length = reader.read_u32::<BigEndian>()?;
        let mut idx_to_node = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let x = reader.read_f64::<BigEndian>()?;
            let y = reader.read_f64::<BigEndian>()?;
            let location = Location(x, y);
            let connected_len = reader.read_u8()?;
            let mut vec = Vec::with_capacity(connected_len as usize);
            for _ in 0..connected_len {
                let idx = reader.read_u32::<BigEndian>()?;
                vec.push(idx);
            }
            let node = Node {
                connected: CompactVec::from_vec(vec),
                location,
            };
            idx_to_node.push(node);
        }

        Ok(OpenStreetMap {
            idx_to_node
        })
    }
    pub fn trim(&self) -> OpenStreetMap {
        let mut chosen_from = HashSet::new();
        let mut origin_map = HashMap::new();
        for i in 0..self.idx_to_node.len() { // avoids first
            chosen_from.insert(i as u32);
        }

        while let Some(id) = chosen_from.iter().next().cloned() {
            let mut frontier_old = Vec::new();
            let mut frontier_new = Vec::new();
            let mut elems = Vec::new();
            chosen_from.remove(&id);
            frontier_old.push(id);
            elems.push(id);
            while !frontier_old.is_empty() {
                for frontier_id in frontier_old {
                    let node = self.get(frontier_id);
                    node.connected.iterator().for_each(|&new| {
                        if chosen_from.remove(&new) {
                            frontier_new.push(new);
                            elems.push(new);
                        }
                    })
                }
                frontier_old = frontier_new.drain(0..frontier_new.len()).collect();
            }
            origin_map.insert(id, elems);
        }

        let (_, id_list) = origin_map.into_iter().max_by_key(|(_, v)| v.len()).unwrap();

        println!("combining!");
        let mut counter = 0;


        let old_id_to_new: HashMap<u32, u32> = id_list.iter().map(|&old_id| {
            let counter_local = counter;
            counter += 1;
            (old_id, counter_local)
        }).collect();

        let mut new_nodes = Vec::new();

        for old_id in id_list {
            let node = self.get(old_id);
            let result_vec: Vec<_> = node.connected.iterator().filter_map(|x| {
                old_id_to_new.get(x).cloned()
            }).collect();

            assert_ne!(result_vec.len(), 0);

            let compact = CompactVec::from_vec(result_vec);
            let new_node = Node {
                connected: compact,
                location: node.location,
            };
            new_nodes.push(new_node);
        }

        OpenStreetMap {
            idx_to_node: new_nodes
        }
    }
    pub fn get(&self, id: u32) -> &Node {
        self.idx_to_node.get(id as usize).unwrap()
    }
    pub fn node_count(&self) -> usize {
        self.idx_to_node.len()
    }
    pub fn length_miles(&self, path: &Path) -> f64 {
        let locations = path.ids.iter().map(|&id| self.get(id).location);
        let mut prev_loc = None;
        let mut total = 0.0;
        for loc in locations {
            let dx = match prev_loc {
                Some(prev) => loc.dist2(prev).sqrt() as f64,
                None => 0.0
            };
            prev_loc = Some(loc);
            total += dx;
        }
        total * 68.703
    }

    pub fn next_to_id(&self, from_id: u32) -> CompactVecIterator<'_, u32> {
        self.get(from_id).connected.iterator()
    }

    pub fn random(&self) -> (u32, &Node) {
        let rng = &mut rand::thread_rng();
        let idx = rng.gen_range(0, self.idx_to_node.len());
        (idx as u32, &self.idx_to_node[idx])
    }
    pub fn closest(&self, lat: f64, long: f64) -> Option<ClosestResult> {
        let mut min_id = None;
        let mut min_val = f64::MAX;
        self.idx_to_node.iter().enumerate().for_each(|(id, node)| {
            if node.connected.len() == 0 { // if there are no direct connections
                return;
            }
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

    #[inline]
    fn valid_way(way: &osmpbf::Way) -> bool {
        way.tags().into_iter().any(|(key, _)| key == "highway")
    }

    pub fn parse_highway_nodes(name: &str) -> Result<HashSet<i64>, io::Error> {
        let reader = ElementReader::from_path(name)?;
        let mut valid_nodes = HashSet::new();

        reader.for_each(|x| if let osmpbf::Element::Way(way) = x {
            if OpenStreetMap::valid_way(&way) {
                for r in way.refs() {
                    valid_nodes.insert(r);
                }
            }
        })?;

        Ok(valid_nodes)
    }

    pub fn parse(name: &str) -> Result<OpenStreetMap, io::Error> {
        let valid = OpenStreetMap::parse_highway_nodes(name)?;
        let mut id_to_idx = HashMap::new();
        let mut idx_to_node = Vec::new();

        let reader = ElementReader::from_path(name)?;

        reader.for_each(|element| {
            if let Some((id, lat, lon)) = match &element {
                osmpbf::Element::Node(n) => Some((n.id(), n.lat(), n.lon())),
                osmpbf::Element::DenseNode(n) => Some((n.id, n.lat(), n.lon())),
                _ => None
            } {
                if valid.contains(&id) {
                    id_to_idx.insert(id, idx_to_node.len() as u32);
                    let location = Location(lat, lon);
                    let to_insert = Node {
                        location,
                        connected: CompactVec::empty(),
                    };
                    idx_to_node.push(to_insert);
                }
            } else if let osmpbf::Element::Way(way) = &element {
                process_way(&mut id_to_idx, &mut idx_to_node, &way)
            }
        })?;

        // prune


        Ok(OpenStreetMap {
            idx_to_node,
        })
    }
}
