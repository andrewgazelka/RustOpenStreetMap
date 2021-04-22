pub mod bi_astar;
mod middleman;
mod path_constructor;

pub struct SimpleNode {
    on: u32,
    parent: u32,
}
