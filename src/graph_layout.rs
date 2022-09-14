use std::{collections::HashMap, option::Iter, sync::Arc};

use druid::Vec2;
use itertools::Itertools;
use rand::prelude::*;

use crate::block::{Block, CodeFlow};

#[derive(Default, Debug, Clone)]
pub struct LinkAttributes {
    pub count: usize,
}
#[derive(Default, Debug, Clone)]
pub struct PositionedPath {
    points: Vec<Vec2>,
}

#[derive(Default, Debug, Clone)]
pub struct Link {
    origin: usize,
    destination: usize,
    pub attributes: LinkAttributes,
}
#[derive(Default, Debug, Clone)]
pub struct PositionedLink {
    pub raw_link: Link,
    path: PositionedPath,
}
#[derive(Debug, Clone)]
pub struct PositionedNode {
    pub val: Arc<Block>,
    pub position: Vec2,
    velocity: Vec2,
}

const REPEL_FORCE: f64 = 0.10; // d^2
const ATTRACT_FORCE: f64 = 0.20; // d
const MIN_ATTRACT_DISTANCE: f64 = 5.0;

const AREA_WIDTH: f64 = 1.0;
const AREA_HEIGHT: f64 = 1.0;

#[derive(Debug)]
pub struct GraphLayout {
    pub raw_flow: CodeFlow,
    pub links: HashMap<(usize, usize), PositionedLink>,
    pub nodes: HashMap<usize, PositionedNode>,
}

// trait Searchable<'a,K,T> {
//     fn find(&self,val:K)->&T;
//     fn get_connections(&self,val:&T) -> Vec<Link<K>>;
//     fn iter(&self) -> Iter<'a,T>;
// }

impl GraphLayout {
    pub fn new(data: CodeFlow) -> GraphLayout {
        let mut links: HashMap<(usize, usize), PositionedLink> = HashMap::new();
        let mut nodes = HashMap::new();
        let len = data.blocks.len();
        let mut i = 0;
        let mut rng = rand::thread_rng();
        for node_data in data.blocks.iter() {
            nodes.insert(
                node_data.val.base().clone(),
                PositionedNode {
                    val: node_data.val.clone(),
                    position: Vec2 {
                        x:rng.gen(),
                        y:rng.gen(),
                        // x: ((i %2) as f64 / (2. * len as f64)),
                        // y: (i as f64) / (len as f64),
                    },
                    velocity: Vec2::default(),
                },
            );
            i += 1;
        }
        for (start_path, end) in data.path.iter().tuple_windows() {
            let block_base = data
                .blocks
                .find(*start_path, *start_path)
                .next()
                .unwrap()
                .val
                .base();
            let block_end = data
                .blocks
                .find(*end, *end)
                .next()
                .unwrap()
                .val
                .base();
            let kvp = (*block_base, *block_end);
            if let Some(link) = links.get_mut(&kvp) {
                link.raw_link.attributes.count += 1;
            } else {
                links.insert(kvp, {
                    PositionedLink {
                        raw_link: Link {
                            origin: *block_base,
                            destination: *end,
                            attributes: LinkAttributes { count: 1 },
                        },
                        path: PositionedPath { points: Vec::new() },
                    }
                });
            }
        }
        let mut layout = GraphLayout {
            raw_flow: data,
            links,
            nodes,
        };
        layout.force_position(500);
        layout.normalize();
        // for k in layout.nodes.values() {
        //     dbg!(k.position);
        // }
        // dbg!(&layout);
        println!("Unique links: {}",layout.links.len());
        layout
    }
    pub fn force_position(&mut self, num_iter: usize) {
        for _ in 0..num_iter {
            // all this because of fucking concurrent modification
            // You would think this would be something that would be easy to
            // do in a language that cares so much about mutability.
            let mut forces: HashMap<usize, Vec2> = HashMap::new();
            for k in self.nodes.keys() {
                forces.insert(*k, Vec2::default());
            }
            for (a, b) in self.nodes.values().tuple_combinations() {
                let d_vec = a.position - b.position;
                let d_mag_2 = d_vec.x * d_vec.x + d_vec.y * d_vec.y;
                let d_norm = d_vec.normalize();
                let force = d_norm * REPEL_FORCE / d_mag_2;
                *forces.get_mut(a.val.base()).unwrap() += force;
                *forces.get_mut(b.val.base()).unwrap() -= force;
                // pair[0].applied_force += force;
            }
            // attraction_forces
            for ((from,to), link) in &self.links {
                if from==to {
                    continue;
                }
                let a = self.nodes.get(from).unwrap();
                let b = self.nodes.get(to).unwrap();
                let d_vec = a.position - b.position;
                let d_mag_2 = d_vec.x * d_vec.x + d_vec.y * d_vec.y;
                let d_mag = d_mag_2.sqrt();
                assert!(d_mag>0.00001);
                if d_mag<MIN_ATTRACT_DISTANCE {
                    continue;
                }
                let d_norm = d_vec.normalize();
                let force = d_norm * (link.raw_link.attributes.count as f64) * ATTRACT_FORCE / d_mag;
                *forces.get_mut(a.val.base()).unwrap() -= force;
                *forces.get_mut(b.val.base()).unwrap() += force;

            }

            self.apply_forces(forces);
            self.apply_velocities();
        }
    }
    fn normalize(&mut self) {
        let (mut min_x, mut min_y) = (100000.0_f64, 100000.0_f64);
        let (mut max_x, mut max_y) = (-100000.0_f64, -100000.0_f64);
        for node in self.nodes.values() {
            if node.position.x > max_x {
                max_x = node.position.x;
            }
            if node.position.x < min_x {
                min_x = node.position.x;
            }
            if node.position.y > max_y {
                max_y = node.position.y;
            }
            if node.position.y < min_y {
                min_y = node.position.y;
            }
        }
        // set 0,0 as minimum and 1,1 as maximum
        for node in self.nodes.values_mut() {
            if min_x < 0. {
                node.position.x += min_x.abs();
                max_x += min_x.abs();
            }
            if min_y < 0. {
                node.position.y += min_y.abs();
                max_y += min_y.abs();
            }
            let max_dim = if max_x > max_y { max_x } else { max_y };
            node.position.x /= max_x;
            node.position.y /= max_y;

            // }else {
            //     node.position[0]+=min_x;
            //     node.position[0]*=max_dim;
            //     node.position[1]+=min_y;
            //     node.position[1]*=max_dim;

            // }
        }
    }
    fn apply_velocities(&mut self) {
        for node in self.nodes.values_mut() {
            node.position.x += node.velocity.x;
            node.position.y += node.velocity.y;
        }
    }
    fn apply_forces(&mut self, forces: HashMap<usize, Vec2>) {
        for node in self.nodes.values_mut() {
            let force = forces.get(node.val.base()).unwrap();
            node.velocity += *force;
        }
    }
}
