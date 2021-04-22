use std::collections::{BinaryHeap, HashMap};
use std::sync::mpsc::Sender;

use crate::a_star::{HeapNode, Path};
use crate::bidirectional::middleman::Middleman;
use crate::bidirectional::path_constructor::PathConstructor;
use crate::osm_parser;
use crate::osm_parser::OpenStreetMap;

pub fn a_star_bi(map: &osm_parser::OpenStreetMap, init_node: u32, goal_node: u32) -> Option<Path> {
    let middleman = Middleman::new();

    let sender1 = middleman.node_sender.clone();
    let sender2 = middleman.node_sender.clone();

    let mut forward = None;
    let mut backward= None;

    rayon::scope(|scope| {
        scope.spawn(|_| {
            forward= Some(bi_path_helper(map, init_node, goal_node, sender1));
        });

        scope.spawn(|_| {
            backward= Some(bi_path_helper(map, goal_node, init_node, sender2));
        });
    });

    let (forward,backward) = (forward.unwrap(), backward.unwrap());

    if let Some(split) = middleman.get_split() {
        let ids = PathConstructor::build_path(&forward, &backward, split);

        return Some(Path {
            ids,
            parent_map: map,
        });
    }

    None
}


fn bi_path_helper(map: &OpenStreetMap, init_node: u32, goal_node: u32, node_sender: Sender<u32>) -> HashMap<u32, u32> {

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
            return track; // Some(construct_path(origin.id, &track, map));
        }

        let origin_id = &origin.id;
        let origin_g_score = g_scores[&origin_id];

        let origin_loc = map.get(*origin_id).location;

        for neighbor in map.next_to_id(origin.id) {
            let neighbor_node = map.get(*neighbor);
            let neighbor_loc = neighbor_node.location;
            let tentative_g_score = origin_g_score + neighbor_loc.dist2(origin_loc);
            match g_scores.get_mut(&neighbor) {
                Some(prev_score) => if tentative_g_score < *prev_score {
                    *prev_score = tentative_g_score;
                } else {
                    continue;
                }
                None => {
                    g_scores.insert(*neighbor, tentative_g_score);
                }
            };


            if track.insert(*neighbor, *origin_id).is_none() { // if this is the first time we added to the map

                let send_result = node_sender.send(*neighbor);

                // this will be an error if the send channel has been closed (which means the middle man has found a collision), so we can stop
                if send_result.is_err() {
                    return track;
                }
            }


            let h_score = goal_loc.dist2(neighbor_loc);
            let f_score = tentative_g_score + h_score;

            queue.push(HeapNode {
                id: *neighbor,
                f_score,
            })
        }
    }
    track
}
