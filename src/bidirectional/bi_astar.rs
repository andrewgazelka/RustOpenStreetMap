use std::collections::{BinaryHeap, HashMap};
use crate::a_star::{HeapNode, Path};
use crate::bidirectional::middleman::Middleman;
use crate::bidirectional::SimpleNode;
use crate::osm_parser::OpenStreetMap;
use crate::osm_parser;
use std::sync::mpsc::Sender;

pub fn a_star_bi(map: &osm_parser::OpenStreetMap, init_node: u32, goal_node: u32) -> Option<Path> {

    let middleman = Middleman::new();

    let sender = &middleman.node_sender;
    let sender1 = sender.clone();
    let sender2 = sender.clone();

    let cb = crossbeam::scope(|scope| {
        scope.spawn(move |_|{
            bi_path_helper(map, init_node, goal_node, sender1);
        });

        scope.spawn(move |_|{
            bi_path_helper(map, goal_node, init_node, sender2);
        });
    });

    if cb.is_err() {
        return None;
    }

    middleman.get_result().map(|ids| Path {
            ids,
            parent_map: map,
        })

}


fn bi_path_helper(map: &OpenStreetMap, init_node: u32, goal_node: u32, node_sender: Sender<SimpleNode>) {

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
            return; // Some(construct_path(origin.id, &track, map));
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

            let send_result = node_sender.send(SimpleNode {
                parent: *origin_id,
                on: *neighbor,
            });

            if send_result.is_err() { return }

            let h_score = goal_loc.dist2(neighbor_loc);
            let f_score = tentative_g_score + h_score;

            queue.push(HeapNode {
                id: *neighbor,
                f_score,
            })
        })
    }

    return
}
