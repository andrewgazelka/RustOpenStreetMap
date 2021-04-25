use crate::osm_parser::Node;

pub trait Params<T> : std::marker::Sync {
    fn heuristic(&self, on: &T, goal: &T) -> f64;
    fn neighbor_dist(&self, on: &T, next: &T) -> f64;
}




pub struct SimpleParams;

impl Params<Node> for SimpleParams {
    fn heuristic(&self, on: &Node, goal: &Node) -> f64 {
        on.location.dist(goal.location)
    }

    fn neighbor_dist(&self, on: &Node, next: &Node) -> f64 {
        on.location.dist(next.location)
    }
}
