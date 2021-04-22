use std::sync::mpsc::{Sender, Receiver, SendError};
use crate::bidirectional::SimpleNode;
use std::sync::mpsc;
use std::thread;
use crate::bidirectional::path_constructor::PathConstructor;
use std::collections::HashSet;

pub struct Middleman {
    pub node_sender: Sender<SimpleNode>,
    pub vec_receiver: Receiver<Vec<u32>>,
}

impl Middleman {
    pub fn new() -> Middleman {
        let (send_node, receive_node) = mpsc::channel();
        let (send_vec, receive_vec) = mpsc::channel();
        thread::spawn(move || {
            let mut constructor = PathConstructor::new();

            for elem in receive_node {
                let res = constructor.attempt_path(elem);
                if let Some(v) = res {
                    send_vec.send(v);
                    break;
                }
            }
        });

        Middleman {
            node_sender: send_node,
            vec_receiver: receive_vec,
        }
    }

    pub fn send(&self, node: SimpleNode) -> Result<(), SendError<SimpleNode>> {
        return self.node_sender.send(node)
    }

    pub fn get_result(&self) -> Option<Vec<u32>> {
        match self.vec_receiver.recv() {
            Ok(path) => Some(path),
            Err(_) => None
        }
    }

}
