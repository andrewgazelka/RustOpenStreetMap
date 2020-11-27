use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};

use crate::osm_parser::{Node, OpenStreetMap};

struct HeapNode {
    id: i64,
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

fn construct_path(init: i64, map: &HashMap<i64, i64>) -> Vec<i64> {
    let mut vec = Vec::new();
    let mut on = &init;
    vec.push(*on);
    while let Some(prev) = map.get(on) {
        vec.push(*prev);
        on = prev;
    }
    vec.reverse();
    vec
}

pub fn path(map: &OpenStreetMap, init_node: Node, goal_node: Node) -> Option<Vec<i64>> {
    let goal_loc = &goal_node.location;

    // also is an explored
    let mut g_scores = HashMap::new();
    let mut queue = BinaryHeap::new();

    let mut track = HashMap::new();

    // init
    g_scores.insert(init_node.id, 0f64);

    queue.push(HeapNode {
        id: init_node.id,
        f_score: f64::MAX,
    });

    while let Some(origin) = queue.pop() {
        // let origin_node = origin.node;
        if origin.id == goal_node.id {
            return Some(construct_path(origin.id, &track));
        }

        let origin_id = &origin.id;
        let origin_g_score = g_scores[&origin_id];

        let origin_loc = &map.node_map[origin_id];

        map.next_to_id(origin.id).for_each(|neighbor| {
            let neighbor_loc = &neighbor.location;
            let tentative_g_score = origin_g_score + neighbor_loc.dist2(origin_loc);
            match g_scores.get_mut(&neighbor.id) {
                Some(prev_score) => if tentative_g_score < *prev_score {
                    // println!("less by {}", *prev_score - tentative_g_score);

                    *prev_score = tentative_g_score;
                } else {
                    return;
                }
                None => {
                    g_scores.insert(neighbor.id, tentative_g_score);
                }
            };


            track.insert(neighbor.id, origin.id);

            let h_score = goal_loc.dist2(neighbor_loc);
            let f_score = tentative_g_score + h_score;

            queue.push(HeapNode {
                id: neighbor.id,
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
        // queue.push(7);
        // queue.push(3);
        // queue.push(9);
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
