use std::collections::{BinaryHeap, HashMap, HashSet};

use super::AppData;

pub(crate) const COLLISION_MARGIN: f32 = 2.;

#[derive(Debug, Clone, Copy)]
struct SearchNode {
    id: usize,
    cost: f32,
    came_from: Option<usize>,
}

impl std::cmp::PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl std::cmp::Eq for SearchNode {}

impl std::cmp::PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.cost.partial_cmp(&self.cost)
    }
}

impl std::cmp::Ord for SearchNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

pub struct VisitedNode {
    pub cost: f32,
    pub came_from: Option<usize>,
}

impl VisitedNode {
    pub fn new(cost: f32, came_from: Option<usize>) -> Self {
        Self { cost, came_from }
    }
}

pub type VisitedMap = HashMap<usize, VisitedNode>;

impl AppData {
    pub(super) fn search(&mut self) -> Result<(), String> {
        if let &[ref first, ref second, ..] = &self.con_rects[..] {
            let mut visited = VisitedMap::new();
            let mut next_set = BinaryHeap::new();
            let start_ids = first.connectors();
            let goal_ids = second.connectors();
            self.start_nodes = start_ids.clone();
            for start_id in start_ids {
                next_set.push(SearchNode {
                    id: start_id,
                    cost: 0.,
                    came_from: None,
                });
                visited.insert(start_id, VisitedNode::new(0., None));
            }

            self.goal_nodes = goal_ids.clone();

            let mut obstructed = HashSet::new();
            for rect in self.con_rects.iter() {
                for (j, pt) in self.grid.points.iter().enumerate() {
                    if rect.x - COLLISION_MARGIN <= pt.pos.x
                        && pt.pos.x < rect.x + rect.width + COLLISION_MARGIN
                        && rect.y - COLLISION_MARGIN <= pt.pos.y
                        && pt.pos.y < rect.y + rect.height + COLLISION_MARGIN
                    {
                        obstructed.insert(j);
                    }
                }
            }

            // println!("Obstructed: {obstructed:?}");

            let mut iter = 0;

            while let Some(s_node) = next_set.pop() {
                if self.goal_nodes.iter().any(|goal| *goal == s_node.id) {
                    let mut path = vec![s_node.id];
                    let mut prev = s_node.came_from;
                    while let Some(came_from) = prev {
                        path.push(came_from);
                        prev = visited.get(&came_from).and_then(|node| node.came_from);
                        iter += 1;
                        if 1000 < iter {
                            self.visited_nodes = Some(visited);
                            return Err("Path find iteration exceeds 1000".to_string());
                        }
                    }
                    println!("Path found! {path:?}");
                    self.visited_nodes = Some(visited);
                    self.path = Some(path);
                    return Ok(());
                }
                let this_node = self.grid.points[s_node.id].pos;
                let node = &self.grid.points[s_node.id];
                for con in &node.connect {
                    if obstructed.contains(con) {
                        continue;
                    }
                    let new_node = self.grid.points[*con].pos;
                    let new_cost = s_node.cost + this_node.distance(new_node);
                    visited
                        .entry(*con)
                        .and_modify(|e| {
                            if new_cost < e.cost {
                                e.cost = new_cost;
                                e.came_from = Some(s_node.id);
                                let new_node = SearchNode {
                                    id: *con,
                                    cost: new_cost,
                                    came_from: Some(s_node.id),
                                };
                                // println!("Adding {new_node:?}");
                                next_set.push(new_node);
                            }
                        })
                        .or_insert_with(|| {
                            let new_node = SearchNode {
                                id: *con,
                                cost: new_cost,
                                came_from: Some(s_node.id),
                            };
                            // println!("Adding {new_node:?}");
                            next_set.push(new_node);
                            VisitedNode::new(new_cost, Some(s_node.id))
                        });
                }
                iter += 1;
                if 1000 < iter {
                    self.visited_nodes = Some(visited);
                    return Err("Exceed 1000 iterations".to_string());
                }
            }
        }
        self.visited_nodes = None;
        Ok(())
    }
}
