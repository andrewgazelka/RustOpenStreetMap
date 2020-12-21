use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::osm_parser::OpenStreetMap;

struct HeapNode {
    id: u32,
    f_score: f64,
}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.f_score.partial_cmp(&self.f_score)
    }
}

impl PartialEq for HeapNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_score.eq(&other.f_score)
    }
}

impl Eq for HeapNode {}


fn construct_path<'a>(init: u32, map: &HashMap<u32, u32>, osm: &'a OpenStreetMap) -> Path<'a> {
    let mut ids = Vec::new();
    let mut on = &init;
    ids.push(*on);
    while let Some(prev) = map.get(on) {
        ids.push(*prev);
        on = prev;
    }
    ids.reverse();
    Path { ids, parent_map: osm }
}

pub struct Path<'a> {
    pub ids: Vec<u32>,
    pub parent_map: &'a OpenStreetMap,
}

impl <'a> Path<'a> {
    pub fn length_miles(&self) -> f64 {
        let map = self.parent_map;
        let locations = self.ids.iter().map(|&id| map.get(id).location);
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
}


#[allow(dead_code)]
pub fn path(map: &OpenStreetMap, init_node: u32, goal_node: u32) -> Option<Path> {

    // also is an explored
    let mut g_scores = HashMap::new();
    let mut queue = BinaryHeap::new();

    let mut track = HashMap::new();

    // init
    g_scores.insert(init_node, 0f64);

    let goal_loc = map.get(goal_node).location;

    queue.push(HeapNode {
        id: init_node,
        f_score: f64::MAX,
    });

    while let Some(origin) = queue.pop() {
        // let origin_node = origin.node;
        if origin.id == goal_node {
            return Some(construct_path(origin.id, &track, map));
        }

        let origin_id = &origin.id;
        let origin_g_score = g_scores[&origin_id];

        let origin_loc = map.get(*origin_id).location;

        map.next_to_id(origin.id).for_each(|neighbor| {
            let neighbor_node = map.get(*neighbor);
            let neighbor_loc = neighbor_node.location;
            let tentative_g_score = origin_g_score + neighbor_loc.dist2(origin_loc);
            match g_scores.get_mut(&neighbor) {
                Some(prev_score) => if tentative_g_score < *prev_score {
                    *prev_score = tentative_g_score;
                } else {
                    return;
                }
                None => {
                    g_scores.insert(*neighbor, tentative_g_score);
                }
            };

            track.insert(*neighbor, *origin_id);

            let h_score = goal_loc.dist2(neighbor_loc);
            let f_score = tentative_g_score + h_score;

            queue.push(HeapNode {
                id: *neighbor,
                f_score,
            })
        })
    }

    None
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use crate::a_star::HeapNode;

    #[test]
    fn queue_min() {
        let mut queue = BinaryHeap::new();
        queue.push(HeapNode {
            id: 1,
            f_score: 7f64,
        });

        queue.push(HeapNode {
            id: 2,
            f_score: 3f64,
        });

        queue.push(HeapNode {
            id: 3,
            f_score: 9f64,

        });
        assert_eq!(2, queue.pop().unwrap().id);
        assert_eq!(1, queue.pop().unwrap().id);
        assert_eq!(3, queue.pop().unwrap().id);
    }
}
