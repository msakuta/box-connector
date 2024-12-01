use eframe::egui::{pos2, Pos2};

use crate::ConRect;

#[derive(Debug, Clone)]
pub(crate) struct GridPoint {
    pub pos: Pos2,
    pub connect: Vec<usize>,
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

        for rect in con_rects {
            insert_interval(&mut intervals_x, rect.x);
            insert_interval(&mut intervals_x, rect.x + rect.width / 2.);
            insert_interval(&mut intervals_x, rect.x + rect.width);

            insert_interval(&mut intervals_y, rect.y);
            insert_interval(&mut intervals_y, rect.y + rect.height / 2.);
            insert_interval(&mut intervals_y, rect.y + rect.height);
        }

        let y_len = intervals_y.len();

        let borrow_intervals_x = &intervals_x;

        let points = intervals_y
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
