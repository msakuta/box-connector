use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::grid::find_rect_node;

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

impl AppData {
    pub(super) fn search(&mut self) -> Result<(), String> {
        if let &[ref first, ref second, ..] = &self.con_rects[..] {
            let mut visited = HashMap::new();
            let mut next_set = BinaryHeap::new();
            let mut start_ids = vec![];
            if let Some(left_con) = first.left_con {
                start_ids.push(left_con);
            }
            if let Some(right_con) = first.right_con {
                start_ids.push(right_con);
            }
            if start_ids.is_empty() {
                if let Some(id) = find_rect_node(&self.grid.points, first) {
                    start_ids.push(id);
                }
            }
            let goal_id = second
                .left_con
                .or_else(|| find_rect_node(&self.grid.points, second));
            self.start_nodes = start_ids.clone();
            for start_id in start_ids {
                next_set.push(SearchNode {
                    id: start_id,
                    cost: 0.,
                    came_from: None,
                });
                visited.insert(start_id, (0., None));
            }

            self.goal = goal_id;

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
                if Some(s_node.id) == self.goal {
                    let mut path = vec![s_node.id];
                    let mut prev = s_node.came_from;
                    while let Some(came_from) = prev {
                        path.push(came_from);
                        prev = visited.get(&came_from).and_then(|(_, prev)| *prev);
                        iter += 1;
                        if 1000 < iter {
                            return Err("Path find iteration exceeds 1000".to_string());
                        }
                    }
                    println!("Path found! {path:?}");
                    self.path = Some(path);
                    break;
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
                            if s_node.cost < e.0 {
                                e.0 = new_cost;
                                e.1 = Some(s_node.id);
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
                            (new_cost, Some(s_node.id))
                        });
                }
                iter += 1;
                if 1000 < iter {
                    return Err("Exceed 1000 iterations".to_string());
                }
            }
        }
        Ok(())
    }
}
