use eframe::egui::{pos2, Pos2};

use crate::{search::COLLISION_MARGIN, ConRect};

#[derive(Debug, Clone)]
pub(crate) struct GridPoint {
    pub pos: Pos2,
    pub connect: Vec<usize>,
}

impl GridPoint {
    pub fn new(pos: Pos2, connect: Vec<usize>) -> Self {
        Self { pos, connect }
    }
}

pub(crate) struct Grid {
    pub intervals_x: Vec<f32>,
    pub intervals_y: Vec<f32>,
    pub points: Vec<GridPoint>,
}

impl Grid {
    pub(super) fn new(con_rects: &mut [ConRect]) -> Self {
        let mut intervals_x: Vec<_> = (0..10).map(|x| (x * 100) as f32).collect();
        let mut intervals_y: Vec<_> = (0..10).map(|x| (x * 100) as f32).collect();

        for rect in &mut *con_rects {
            // insert_interval(&mut intervals_x, rect.x);
            insert_interval(&mut intervals_x, rect.x + rect.width / 2.);
            // insert_interval(&mut intervals_x, rect.x + rect.width);

            // insert_interval(&mut intervals_y, rect.y);
            insert_interval(&mut intervals_y, rect.y + rect.height / 2.);
            // insert_interval(&mut intervals_y, rect.y + rect.height);
        }

        let y_len = intervals_y.len();

        let borrow_intervals_x = &intervals_x;

        let mut points: Vec<_> = intervals_y
            .iter()
            .enumerate()
            .map(|(iy, y)| {
                let y = *y;
                borrow_intervals_x.iter().enumerate().map(move |(ix, x)| {
                    let mut connect = vec![];
                    if 0 < ix {
                        connect.push(ix - 1 + iy * y_len);
                    }
                    if ix < borrow_intervals_x.len() - 1 {
                        connect.push(ix + 1 + iy * y_len);
                    }
                    if 0 < iy {
                        connect.push(ix + (iy - 1) * y_len);
                    }
                    if iy < y_len - 1 {
                        connect.push(ix + (iy + 1) * y_len);
                    }
                    GridPoint {
                        pos: pos2(*x, y),
                        connect,
                    }
                })
            })
            .flatten()
            .collect();

        for rect in con_rects {
            let pos = pos2(rect.x - COLLISION_MARGIN * 2., rect.y + rect.height / 2.);
            rect.left_con = insert_horz_intersection(&mut points, pos);
            let pos = pos2(
                rect.x + rect.width + COLLISION_MARGIN * 2.,
                rect.y + rect.height / 2.,
            );
            rect.right_con = insert_horz_intersection(&mut points, pos);
            let pos = pos2(rect.x + rect.width / 2., rect.y - COLLISION_MARGIN * 2.);
            rect.top_con = insert_vert_intersection(&mut points, pos);
            let pos = pos2(
                rect.x + rect.width / 2.,
                rect.y + rect.height + COLLISION_MARGIN * 2.,
            );
            rect.bottom_con = insert_vert_intersection(&mut points, pos);
        }

        Self {
            intervals_x,
            intervals_y,
            points,
        }
    }
}

fn insert_interval(intervals: &mut Vec<f32>, pos: f32) {
    // Stupid linear search, because binary_search won't work with f32
    let res = intervals
        .iter()
        .enumerate()
        .find(|(_, x)| pos < **x)
        .map(|(i, _)| i);
    if let Some(res) = res {
        if pos != intervals[res] {
            intervals.insert(res, pos);
        }
    }
}

struct ConIdx {
    node_id: usize,
    con_idx: usize,
}

fn find_horz_intersection(points: &[GridPoint], pos: Pos2) -> Option<(ConIdx, ConIdx)> {
    for (i, point) in points.iter().enumerate() {
        if 1. < (pos.y - point.pos.y).abs() {
            continue;
        }
        for (j, con) in point.connect.iter().enumerate() {
            let point2 = &points[*con];
            if 1. < (pos.y - point2.pos.y).abs() {
                continue;
            }
            let intersecting = if point2.pos.x < point.pos.x {
                point2.pos.x < pos.x && pos.x < point.pos.x
            } else {
                point.pos.x < pos.x && pos.x < point2.pos.x
            };

            if intersecting {
                return Some((
                    ConIdx {
                        node_id: i,
                        con_idx: j,
                    },
                    ConIdx {
                        node_id: *con,
                        con_idx: point2
                            .connect
                            .iter()
                            .enumerate()
                            .find(|(_, v)| **v == i)
                            .map(|(i, _)| i)?,
                    },
                ));
            }
        }
    }
    None
}

fn insert_horz_intersection(points: &mut Vec<GridPoint>, pos: Pos2) -> Option<usize> {
    if let Some((from, to)) = find_horz_intersection(points, pos) {
        let inserted_node = GridPoint::new(pos2(pos.x, pos.y), vec![from.node_id, to.node_id]);
        let inserted_id = points.len();
        points.push(inserted_node);
        points[from.node_id].connect[from.con_idx] = inserted_id;
        points[to.node_id].connect[to.con_idx] = inserted_id;
        Some(inserted_id)
    } else {
        None
    }
}

fn find_intersection(
    points: &[GridPoint],
    pos: Pos2,
    scan_axis: impl Fn(Pos2) -> f32,
    fixed_axis: impl Fn(Pos2) -> f32,
) -> Option<(ConIdx, ConIdx)> {
    for (i, point) in points.iter().enumerate() {
        if 1. < (fixed_axis(pos) - fixed_axis(point.pos)).abs() {
            continue;
        }
        let pos_scan = scan_axis(pos);
        let point_scan = scan_axis(point.pos);
        for (j, con) in point.connect.iter().enumerate() {
            let point2 = &points[*con];
            if 1. < (fixed_axis(pos) - fixed_axis(point2.pos)).abs() {
                continue;
            }
            let point2_scan = scan_axis(point2.pos);
            let intersecting = if point2_scan < point_scan {
                point2_scan < pos_scan && pos_scan < point_scan
            } else {
                point_scan < pos_scan && pos_scan < point2_scan
            };

            if intersecting {
                return Some((
                    ConIdx {
                        node_id: i,
                        con_idx: j,
                    },
                    ConIdx {
                        node_id: *con,
                        con_idx: point2
                            .connect
                            .iter()
                            .enumerate()
                            .find(|(_, v)| **v == i)
                            .map(|(i, _)| i)?,
                    },
                ));
            }
        }
    }
    None
}

fn insert_vert_intersection(points: &mut Vec<GridPoint>, pos: Pos2) -> Option<usize> {
    if let Some((from, to)) = find_intersection(points, pos, |pos| pos.y, |pos| pos.x) {
        let inserted_node = GridPoint::new(pos2(pos.x, pos.y), vec![from.node_id, to.node_id]);
        let inserted_id = points.len();
        points.push(inserted_node);
        points[from.node_id].connect[from.con_idx] = inserted_id;
        points[to.node_id].connect[to.con_idx] = inserted_id;
        Some(inserted_id)
    } else {
        None
    }
}
