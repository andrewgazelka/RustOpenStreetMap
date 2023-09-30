use std::collections::HashMap;
pub struct PathConstructor;

fn path_trace(from: u32, lookup: &HashMap<u32, u32>, into: &mut Vec<u32>) {
    let mut on = from;
    into.push(on);
    while let Some(&prev) = lookup.get(&on) {
        into.push(prev);
        on = prev;
    }
}

impl PathConstructor {
    pub fn build_path(
        forward: &HashMap<u32, u32>,
        backward: &HashMap<u32, u32>,
        split: u32,
    ) -> Vec<u32> {
        let mut vec = Vec::new();
        path_trace(split, forward, &mut vec);
        vec.reverse();
        path_trace(split, backward, &mut vec);
        vec
    }
}
