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
    pub(super) fn new(con_rects: &[ConRect]) -> Self {
        let mut intervals_x: Vec<_> = (0..10).map(|x| (x * 100) as f32).collect();
        let mut intervals_y: Vec<_> = (0..10).map(|x| (x * 100) as f32).collect();

        for rect in con_rects {
            let x_center = rect.x + rect.width / 2.;

            // Stupid linear search, because binary_search won't work with f32
            let res = intervals_x
                .iter()
                .enumerate()
                .find(|(_, x)| x_center < **x)
                .map(|(i, _)| i);
            if let Some(res) = res {
                if x_center != intervals_x[res] {
                    intervals_x.insert(res, x_center);
                }
            }

            let y_center = rect.y + rect.height / 2.;
            let res = intervals_y
                .iter()
                .enumerate()
                .find(|(_, x)| y_center < **x)
                .map(|(i, _)| i);
            if let Some(res) = res {
                if y_center != intervals_y[res] {
                    intervals_y.insert(res, y_center);
                }
            }
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
