use std::{
    collections::HashSet,
    sync::{
        mpsc,
        mpsc::{Receiver, SendError, Sender},
    },
    thread,
};

use crate::bidirectional::SimpleNode;

pub struct Middleman {
    pub node_sender: Sender<u32>,
    pub vec_receiver: Receiver<u32>,
}

impl Middleman {
    pub fn new() -> Middleman {
        let (send_node, receive_node) = mpsc::channel();
        let (send_vec, receive_vec) = mpsc::channel();

        thread::spawn(move || {
            let mut traversed_set = HashSet::new();

            for elem in receive_node {
                let was_empty = traversed_set.insert(elem);
                if !was_empty {
                    send_vec.send(elem);
                    return;
                }
            }
        });

        Middleman {
            node_sender: send_node,
            vec_receiver: receive_vec,
        }
    }

    pub fn get_split(&self) -> Option<u32> {
        match self.vec_receiver.recv() {
            Ok(split_point) => Some(split_point),
            Err(_) => None,
        }
    }
}
