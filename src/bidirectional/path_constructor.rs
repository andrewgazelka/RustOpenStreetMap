use std::collections::HashMap;
use crate::bidirectional::SimpleNode;
use std::collections::hash_map::Entry;

pub struct PathConstructor {
    map: HashMap<u32, u32>,
}

impl PathConstructor {
    pub fn new() -> PathConstructor {
        PathConstructor {
            map: HashMap::new()
        }
    }

    pub fn attempt_path(&mut self, elem: SimpleNode) -> Option<Vec<u32>> {
        match self.map.entry(elem.on) {
            Entry::Occupied(_) => {
                let mut vec = Vec::new();
                self.path_trace(elem.parent, &mut vec);
                vec.reverse();
                self.path_trace(elem.on, &mut vec);
                return Some(vec);
            }
            Entry::Vacant(x) => {
                x.insert(elem.parent);
            }
        }

        None
    }

    fn path_trace(&self, from: u32, into: &mut Vec<u32>) {
        let mut on = from;
        into.push(on);
        while let Some(&prev) = self.map.get(&on) {
            into.push(prev);
            on = prev;
        }
    }
}
