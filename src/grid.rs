use eframe::egui::{pos2, Pos2};

use crate::ConRect;

#[derive(Debug, Clone)]
pub(crate) struct GridPoint {
    pub pos: Pos2,
    pub connect: Vec<usize>,
}

pub(super) fn gen_grid(con_rects: &[ConRect]) -> (Vec<f32>, Vec<f32>, Vec<GridPoint>) {
    let mut grid_intervals_x: Vec<_> = (0..10).map(|x| (x * 100) as f32).collect();
    let mut grid_intervals_y: Vec<_> = (0..10).map(|x| (x * 100) as f32).collect();

    for rect in con_rects {
        let x_center = rect.x + rect.width / 2.;

        // Stupid linear search, because binary_search won't work with f32
        let res = grid_intervals_x
            .iter()
            .enumerate()
            .find(|(_, x)| x_center < **x)
            .map(|(i, _)| i);
        if let Some(res) = res {
            if x_center != grid_intervals_x[res] {
                grid_intervals_x.insert(res, x_center);
            }
        }

        let y_center = rect.y + rect.height / 2.;
        let res = grid_intervals_y
            .iter()
            .enumerate()
            .find(|(_, x)| y_center < **x)
            .map(|(i, _)| i);
        if let Some(res) = res {
            if y_center != grid_intervals_y[res] {
                grid_intervals_y.insert(res, y_center);
            }
        }
    }

    let y_len = grid_intervals_y.len();

    let intervals_x = &grid_intervals_x;

    let grid_points = grid_intervals_y
        .iter()
        .enumerate()
        .map(|(iy, y)| {
            let y = *y;
            intervals_x.iter().enumerate().map(move |(ix, x)| {
                let mut connect = vec![];
                if 0 < ix {
                    connect.push(ix - 1 + iy * y_len);
                }
                if ix < intervals_x.len() - 1 {
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

    (grid_intervals_x, grid_intervals_y, grid_points)
}
