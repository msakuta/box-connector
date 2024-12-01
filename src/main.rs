mod search;

use eframe::{
    egui::{
        vec2, Align2, CentralPanel, FontId, Frame, Painter, Response, Sense, Shape, SidePanel, Ui,
    },
    emath,
    epaint::{pos2, Color32, Pos2, Rect},
};

fn main() {
    let con_rects = vec![
        ConRect::new(130., 70., 140., 50.),
        ConRect::new(420., 120., 90., 30.),
        ConRect::new(260., 420., 150., 60.),
    ];

    let app_data = AppData::new(con_rects);

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "box-connector application in eframe",
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(App {
                // img: BgImage::new(),
                app_data,
                auto_find_path: false,
                show_grid_label: true,
            }))
        }),
    )
    .unwrap();
}

pub struct App {
    // img: BgImage,
    app_data: AppData,
    show_grid_label: bool,
    auto_find_path: bool,
}

struct AppData {
    con_rects: Vec<ConRect>,
    grid_intervals_x: Vec<f32>,
    grid_intervals_y: Vec<f32>,
    grid_points: Vec<GridPoint>,
    start: Option<usize>,
    goal: Option<usize>,
    path: Option<Vec<usize>>,
    selected_rect: Option<usize>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        SidePanel::right("side_panel")
            .resizable(false)
            .min_width(200.)
            .show(ctx, |ui| {
                if ui.button("Find path").clicked() {
                    self.app_data.search();
                }
                ui.checkbox(&mut self.auto_find_path, "Auto find path");
                ui.checkbox(&mut self.show_grid_label, "Show grid labels");
            });

        CentralPanel::default().show(ctx, |ui| {
            Frame::canvas(ui.style()).show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
                self.draw(ui, &response, &painter);
            });
        });
    }
}

impl App {
    fn draw(&mut self, ui: &mut Ui, response: &Response, painter: &Painter) {
        struct UiResult {
            interact_pos: Option<Pos2>,
            mouse_down: bool,
            mouse_up: bool,
        }

        let ui_result = ui.input(|input| {
            let interact_pos = input.pointer.interact_pos();
            UiResult {
                interact_pos,
                mouse_up: input.pointer.primary_released(),
                mouse_down: input.pointer.primary_pressed(),
            }
        });

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        if let Some(mouse_pos) = ui_result.interact_pos {
            if ui_result.mouse_down {
                for (i, con_rect) in self.app_data.con_rects.iter().enumerate() {
                    let rect_min = to_screen.transform_pos(pos2(con_rect.x, con_rect.y));
                    let rect_max = to_screen.transform_pos(pos2(
                        con_rect.x + con_rect.width,
                        con_rect.y + con_rect.height,
                    ));
                    if rect_min.x < mouse_pos.x
                        && mouse_pos.x < rect_max.x
                        && rect_min.y < mouse_pos.y
                        && mouse_pos.y < rect_max.y
                    {
                        self.app_data.selected_rect = Some(i);
                        break;
                    }
                }
            }

            let mut moved = false;
            if let Some(selected) = self
                .app_data
                .selected_rect
                .and_then(|s| self.app_data.con_rects.get_mut(s))
            {
                let move_pos = to_screen.inverse().transform_pos(mouse_pos)
                    - vec2(selected.width / 2., selected.height / 2.);
                moved = selected.x != move_pos.x || selected.y != move_pos.y;
                selected.x = move_pos.x;
                selected.y = move_pos.y;
            }

            if moved {
                let (grid_x, grid_y, grid_points) = gen_grid(&self.app_data.con_rects);
                self.app_data.grid_intervals_x = grid_x;
                self.app_data.grid_intervals_y = grid_y;
                self.app_data.grid_points = grid_points;
                if self.auto_find_path {
                    self.app_data.search();
                }
            }
        }

        if ui_result.mouse_up {
            self.app_data.selected_rect = None;
        }

        for grid_line in &self.app_data.grid_intervals_x {
            let line = Shape::line_segment(
                [
                    to_screen.transform_pos(pos2(*grid_line, response.rect.top())),
                    to_screen.transform_pos(pos2(*grid_line, response.rect.bottom())),
                ],
                (1., Color32::GRAY),
            );
            painter.add(line);
        }

        for grid_line in &self.app_data.grid_intervals_y {
            let line = Shape::line_segment(
                [
                    to_screen.transform_pos(pos2(response.rect.left(), *grid_line)),
                    to_screen.transform_pos(pos2(response.rect.right(), *grid_line)),
                ],
                (1., Color32::GRAY),
            );
            painter.add(line);
        }

        const MARKER_SIZE: f32 = 4.;

        for (i, grid_point) in self.app_data.grid_points.iter().enumerate() {
            let rect = Rect {
                min: Pos2::new(
                    grid_point.pos.x as f32 - MARKER_SIZE,
                    grid_point.pos.y as f32 - MARKER_SIZE,
                ),
                max: Pos2::new(
                    grid_point.pos.x as f32 + MARKER_SIZE,
                    grid_point.pos.y as f32 + MARKER_SIZE,
                ),
            };

            let color = if Some(i) == self.app_data.start {
                Color32::RED
            } else if Some(i) == self.app_data.goal {
                Color32::GREEN
            } else {
                Color32::GRAY
            };

            painter.rect_stroke(to_screen.transform_rect(rect), 0., (1., color));

            if self.show_grid_label {
                let font = FontId::monospace(10.);
                painter.text(
                    to_screen.transform_pos(grid_point.pos),
                    Align2::CENTER_BOTTOM,
                    format!("{i}"),
                    font,
                    Color32::BLACK,
                );
            }
        }

        if let Some(ref path) = self.app_data.path {
            let path_pos: Vec<_> = path
                .iter()
                .map(|i| to_screen.transform_pos(self.app_data.grid_points[*i].pos))
                .collect();
            let line = Shape::line(path_pos, (1., Color32::RED));
            painter.add(line);
        }

        for (i, con_rect) in self.app_data.con_rects.iter().enumerate() {
            let rect = Rect {
                min: Pos2::new(con_rect.x, con_rect.y),
                max: Pos2::new(con_rect.x + con_rect.width, con_rect.y + con_rect.height),
            };

            let hover = false;

            let color = if self.app_data.selected_rect == Some(i) {
                Color32::RED
            } else if hover {
                Color32::GREEN
            } else {
                Color32::BLUE
            };

            painter.rect_stroke(to_screen.transform_rect(rect), 0., (1., color));
        }
    }
}

impl AppData {
    fn new(con_rects: Vec<ConRect>) -> Self {
        let (grid_intervals_x, grid_intervals_y, grid_points) = gen_grid(&con_rects);
        // dbg!(&grid_points);

        Self {
            con_rects,
            grid_intervals_x,
            grid_intervals_y,
            grid_points,
            start: None,
            goal: None,
            path: None,
            selected_rect: None,
        }
    }
}

fn gen_grid(con_rects: &[ConRect]) -> (Vec<f32>, Vec<f32>, Vec<GridPoint>) {
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

/// Connectable rectangle
struct ConRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ConRect {
    fn new(x: f32, y: f32, width: f32, height: f32) -> ConRect {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone)]
struct GridPoint {
    pos: Pos2,
    connect: Vec<usize>,
}
