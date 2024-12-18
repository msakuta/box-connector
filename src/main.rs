mod con_rect;
mod grid;
mod search;

use eframe::{
    egui::{
        vec2, Align2, CentralPanel, FontId, Frame, Painter, Response, RichText, Sense, Shape,
        SidePanel, Ui,
    },
    emath::{self, RectTransform},
    epaint::{pos2, Color32, Pos2, Rect},
};
use search::VisitedMap;

use crate::{con_rect::ConRect, grid::Grid};

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
        Box::new(move |_cc| Ok(Box::new(App::new(app_data)))),
    )
    .unwrap();
}

pub struct App {
    // img: BgImage,
    app_data: AppData,
    show_grid: bool,
    show_grid_label: bool,
    show_grid_cost: bool,
    show_search_graph: bool,
    auto_find_path: bool,
    error_msg: Option<String>,
}

struct AppData {
    con_rects: Vec<ConRect>,
    grid: Grid,
    start_nodes: Vec<usize>,
    goal_nodes: Vec<usize>,
    path: Option<Vec<usize>>,
    selected_rect: Option<usize>,
    visited_nodes: Option<VisitedMap>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        SidePanel::right("side_panel")
            .resizable(false)
            .min_width(200.)
            .show(ctx, |ui| {
                if let Some(ref e) = self.error_msg {
                    ui.label(RichText::from(format!("Error: {e}")).color(Color32::RED));
                } else {
                    ui.label("No errors");
                };
                if ui.button("Find path").clicked() {
                    if let Err(e) = self.app_data.search() {
                        self.error_msg = Some(e);
                    }
                }
                ui.checkbox(&mut self.auto_find_path, "Auto find path");
                ui.checkbox(&mut self.show_grid, "Show grid");
                ui.checkbox(&mut self.show_grid_label, "Show grid labels");
                ui.checkbox(&mut self.show_grid_cost, "Show grid cost");
                ui.checkbox(&mut self.show_search_graph, "Show search graph");
            });

        CentralPanel::default().show(ctx, |ui| {
            Frame::canvas(ui.style()).show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
                self.draw(ui, &response, &painter);
            });
        });
    }
}

struct UiResult {
    interact_pos: Option<Pos2>,
    hover_pos: Option<Pos2>,
    mouse_down: bool,
    mouse_up: bool,
}

impl App {
    fn new(app_data: AppData) -> Self {
        Self {
            app_data,
            show_grid: false,
            show_grid_label: false,
            show_grid_cost: false,
            show_search_graph: false,
            auto_find_path: true,
            error_msg: None,
        }
    }

    fn draw(&mut self, ui: &mut Ui, response: &Response, painter: &Painter) {
        let ui_result = ui.input(|input| {
            let interact_pos = input.pointer.interact_pos();
            UiResult {
                interact_pos,
                hover_pos: input.pointer.hover_pos(),
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
                self.app_data.grid = Grid::new(&mut self.app_data.con_rects);
                if self.auto_find_path {
                    if let Err(e) = self.app_data.search() {
                        self.error_msg = Some(e);
                    } else {
                        self.error_msg = None;
                    }
                }
            }
        }

        if ui_result.mouse_up {
            self.app_data.selected_rect = None;
        }

        if self.show_grid {
            self.draw_grid(&ui_result, response, painter, &to_screen);
        }

        if let Some(ref path) = self.app_data.path {
            let path_pos: Vec<_> = path
                .iter()
                .filter_map(|i| {
                    Some(to_screen.transform_pos(self.app_data.grid.points.get(*i)?.pos))
                })
                .collect();
            let line = Shape::line(path_pos, (2., Color32::RED));
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

            painter.rect_stroke(to_screen.transform_rect(rect), 0., (2., color));
        }
    }

    fn draw_grid(
        &mut self,
        ui_result: &UiResult,
        response: &Response,
        painter: &Painter,
        to_screen: &RectTransform,
    ) {
        for grid_line in &self.app_data.grid.intervals_x {
            let line = Shape::line_segment(
                [
                    to_screen.transform_pos(pos2(*grid_line, response.rect.top())),
                    to_screen.transform_pos(pos2(*grid_line, response.rect.bottom())),
                ],
                (1., Color32::LIGHT_GRAY),
            );
            painter.add(line);
        }

        for grid_line in &self.app_data.grid.intervals_y {
            let line = Shape::line_segment(
                [
                    to_screen.transform_pos(pos2(response.rect.left(), *grid_line)),
                    to_screen.transform_pos(pos2(response.rect.right(), *grid_line)),
                ],
                (1., Color32::LIGHT_GRAY),
            );
            painter.add(line);
        }

        const MARKER_SIZE: f32 = 4.;

        for (i, grid_point) in self.app_data.grid.points.iter().enumerate() {
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

            let hover = ui_result.hover_pos.is_some_and(|hov| {
                to_screen.transform_pos(grid_point.pos).distance_sq(hov) < MARKER_SIZE.powi(2)
            });

            if hover {
                for j in &grid_point.connect {
                    if let Some(grid_point_j) = self.app_data.grid.points.get(*j) {
                        let line = Shape::line_segment(
                            [
                                to_screen.transform_pos(grid_point_j.pos),
                                to_screen.transform_pos(grid_point.pos),
                            ],
                            (2., Color32::from_rgb(191, 0, 191)),
                        );
                        painter.add(line);
                    }
                }
            }

            let visited = self
                .app_data
                .visited_nodes
                .as_ref()
                .is_some_and(|nodes| nodes.contains_key(&i));

            let color = if hover {
                Color32::BLUE
            } else if visited {
                Color32::DARK_GREEN
            } else if self.app_data.start_nodes.iter().any(|j| i == *j) {
                Color32::RED
            } else if self.app_data.goal_nodes.iter().any(|j| *j == i) {
                Color32::GREEN
            } else {
                Color32::LIGHT_GRAY
            };

            painter.rect_stroke(
                to_screen.transform_rect(rect),
                0.,
                (if hover { 2. } else { 1. }, color),
            );

            if self.show_grid_label {
                let font = FontId::monospace(10.);
                painter.text(
                    to_screen.transform_pos(grid_point.pos),
                    Align2::CENTER_BOTTOM,
                    format!("{i}"),
                    font,
                    if visited {
                        Color32::BLUE
                    } else {
                        Color32::BLACK
                    },
                );
            }
        }

        for (i, node) in self.app_data.visited_nodes.iter().flatten() {
            let to = to_screen.transform_pos(self.app_data.grid.points[*i].pos);
            if self.show_search_graph {
                if let Some(came_from) = node.came_from {
                    let from = to_screen.transform_pos(self.app_data.grid.points[came_from].pos);
                    let mid = (to + from.to_vec2()) / 2.;
                    let line =
                        Shape::line_segment([from, mid], (2., Color32::from_rgb(0, 127, 191)));
                    painter.add(line);
                    let line = Shape::line_segment([mid, to], (3., Color32::from_rgb(0, 127, 191)));
                    painter.add(line);
                }
            }

            if self.show_grid_cost {
                let font = FontId::monospace(10.);
                painter.text(
                    to_screen.transform_pos(to),
                    Align2::CENTER_TOP,
                    format!("{}", node.cost),
                    font,
                    Color32::RED,
                );
            }
        }
    }
}

impl AppData {
    fn new(mut con_rects: Vec<ConRect>) -> Self {
        let grid = Grid::new(&mut con_rects);

        Self {
            con_rects,
            grid,
            start_nodes: vec![],
            goal_nodes: vec![],
            path: None,
            selected_rect: None,
            visited_nodes: None,
        }
    }
}
