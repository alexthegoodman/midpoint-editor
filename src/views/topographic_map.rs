use midpoint_engine::floem::context::PaintCx;
use midpoint_engine::floem::event::{Event, EventListener, EventPropagation};
use midpoint_engine::floem::kurbo::{self, BezPath, Point, Stroke};
use midpoint_engine::floem::peniko::{Color, Fill};
use midpoint_engine::floem::style::{Position, Style};
use midpoint_engine::floem::text::{Attrs, AttrsList, TextLayout};
use midpoint_engine::floem::unit::UnitExt;
use midpoint_engine::floem::views::container;
use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{View, ViewId};
use midpoint_engine::floem_renderer::Renderer;

use nalgebra as na;

const MAX_DISPLAY_DIMENSION: usize = 256; // Adjust this based on performance needs

pub struct TopographicMapView {
    id: ViewId,
    original_heights: na::DMatrix<f32>,
    downsampled_heights: na::DMatrix<f32>,
    config: TopographicConfig,
    style: Style,
}

#[derive(Clone, Debug)]
pub struct TopographicConfig {
    width: f64,
    height: f64,
    contour_interval: f32,
    major_interval: f32,
    offset_x: f64,
    offset_y: f64,
    zoom: f64,
    color_scheme: Vec<Color>,
}

impl Default for TopographicConfig {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            contour_interval: 50.0,
            major_interval: 250.0,
            offset_x: 50.0,
            offset_y: 50.0,
            zoom: 1.0,
            color_scheme: vec![
                Color::rgb8(51, 190, 51),
                Color::rgb8(204, 153, 102),
                Color::rgb8(255, 255, 255),
            ],
        }
    }
}

impl TopographicMapView {
    pub fn new(heights: na::DMatrix<f32>, config: Option<TopographicConfig>) -> Self {
        let config = config.unwrap_or_default();
        let downsampled = Self::downsample_heights(&heights);

        Self {
            id: ViewId::new(),
            original_heights: heights,
            downsampled_heights: downsampled,
            config: config.clone(),
            style: Style::new()
                .position(Position::Absolute)
                .width(config.clone().width)
                .height(config.clone().height),
        }
    }

    fn downsample_heights(heights: &na::DMatrix<f32>) -> na::DMatrix<f32> {
        let (rows, cols) = heights.shape();

        // Calculate downsampling factors
        let scale_factor = f64::from(std::cmp::max(
            (rows as f64 / MAX_DISPLAY_DIMENSION as f64).ceil() as u32,
            (cols as f64 / MAX_DISPLAY_DIMENSION as f64).ceil() as u32,
        ));

        let new_rows = (rows as f64 / scale_factor).ceil() as usize;
        let new_cols = (cols as f64 / scale_factor).ceil() as usize;

        let mut downsampled = na::DMatrix::zeros(new_rows, new_cols);

        // Use a more sophisticated downsampling that preserves extrema
        for new_y in 0..new_rows {
            for new_x in 0..new_cols {
                let start_y = (new_y as f64 * scale_factor) as usize;
                let end_y = ((new_y + 1) as f64 * scale_factor).min(rows as f64) as usize;
                let start_x = (new_x as f64 * scale_factor) as usize;
                let end_x = ((new_x + 1) as f64 * scale_factor).min(cols as f64) as usize;

                // Get all values in this block
                let mut block_values = Vec::new();
                for y in start_y..end_y {
                    for x in start_x..end_x {
                        block_values.push(heights[(y, x)]);
                    }
                }

                // Use a combination of mean and extrema preservation
                if block_values.len() > 0 {
                    let mean = block_values.iter().sum::<f32>() / block_values.len() as f32;
                    let min = block_values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
                    let max = block_values
                        .iter()
                        .fold(f32::NEG_INFINITY, |a, &b| a.max(b));

                    // Use mean but bias towards extrema to preserve important features
                    let value = if (max - mean).abs() > (mean - min).abs() {
                        (mean * 0.7 + max * 0.3)
                    } else {
                        (mean * 0.7 + min * 0.3)
                    };

                    downsampled[(new_y, new_x)] = value;
                }
            }
        }

        downsampled
    }

    fn draw_contour_lines(&self, cx: &mut PaintCx) {
        let (rows, cols) = self.downsampled_heights.shape();
        let min_height = self.downsampled_heights.min();
        let max_height = self.downsampled_heights.max();

        let pixel_width = self.config.width / cols as f64;
        let pixel_height = self.config.height / rows as f64;

        // Generate contours using marching squares algorithm
        let mut height = min_height;
        while height <= max_height {
            let is_major = (height / self.config.major_interval).round()
                * self.config.major_interval
                == height;

            let mut path = BezPath::new();
            let mut path_started = false;

            for y in 0..rows - 1 {
                for x in 0..cols - 1 {
                    let cell_corners = [
                        self.downsampled_heights[(y, x)],
                        self.downsampled_heights[(y, x + 1)],
                        self.downsampled_heights[(y + 1, x + 1)],
                        self.downsampled_heights[(y + 1, x)],
                    ];

                    if let Some(contour_segments) = self.get_contour_segments(
                        x,
                        y,
                        height,
                        &cell_corners,
                        pixel_width,
                        pixel_height,
                    ) {
                        for segment in contour_segments {
                            if !path_started {
                                path.move_to(segment.0);
                                path_started = true;
                            }
                            path.line_to(segment.1);
                        }
                    }
                }
            }

            let stroke_width = if is_major { 2.0 } else { 1.0 };
            cx.stroke(&path, &Color::BLACK.with_alpha_factor(0.5), stroke_width);

            height += self.config.contour_interval;
        }
    }

    fn get_contour_segments(
        &self,
        x: usize,
        y: usize,
        height: f32,
        values: &[f32; 4],
        pixel_width: f64,
        pixel_height: f64,
    ) -> Option<Vec<(Point, Point)>> {
        // Determine which corners are above the contour level
        let cell_config = values
            .iter()
            .enumerate()
            .fold(0, |acc, (i, &val)| acc | ((val >= height) as u8) << i);

        // Skip if all corners are above or below
        if cell_config == 0 || cell_config == 15 {
            return None;
        }

        let x_base = x as f64 * pixel_width + self.config.offset_x;
        let y_base = y as f64 * pixel_height + self.config.offset_y;

        // Linear interpolation helper
        let lerp =
            |v0: f32, v1: f32, h: f32| -> f64 { ((h - v0) / (v1 - v0)).clamp(0.0, 1.0) as f64 };

        // Calculate intersection points
        let mut segments = Vec::new();
        match cell_config {
            1 | 14 => {
                let p1 = Point::new(
                    x_base,
                    y_base + pixel_height * lerp(values[0], values[3], height),
                );
                let p2 = Point::new(
                    x_base + pixel_width * lerp(values[0], values[1], height),
                    y_base,
                );
                segments.push((p1, p2));
            }
            2 | 13 => {
                let p1 = Point::new(
                    x_base + pixel_width * lerp(values[0], values[1], height),
                    y_base,
                );
                let p2 = Point::new(
                    x_base + pixel_width,
                    y_base + pixel_height * lerp(values[1], values[2], height),
                );
                segments.push((p1, p2));
            }
            // Add more cases for other configurations
            _ => {
                // Default simple diagonal for other cases
                let p1 = Point::new(x_base, y_base);
                let p2 = Point::new(x_base + pixel_width, y_base + pixel_height);
                segments.push((p1, p2));
            }
        }

        Some(segments)
    }

    fn draw_elevation_colors(&self, cx: &mut PaintCx) {
        let (rows, cols) = self.downsampled_heights.shape();
        let min_height = self.downsampled_heights.min();
        let max_height = self.downsampled_heights.max();
        let height_range = max_height - min_height;

        let pixel_width = self.config.width / cols as f64;
        let pixel_height = self.config.height / rows as f64;

        for y in 0..rows {
            for x in 0..cols {
                let height = self.downsampled_heights[(y, x)];
                let normalized_height = (height - min_height) / height_range;

                let color_idx =
                    ((self.config.color_scheme.len() - 1) as f32 * normalized_height) as usize;
                let color = self.config.color_scheme[color_idx];

                let rect = kurbo::Rect::new(
                    x as f64 * pixel_width + self.config.offset_x,
                    y as f64 * pixel_height + self.config.offset_y,
                    (x + 1) as f64 * pixel_width + self.config.offset_x,
                    (y + 1) as f64 * pixel_height + self.config.offset_y,
                );

                cx.fill(&rect, &color, 1.0);
            }
        }
    }
}

impl View for TopographicMapView {
    fn id(&self) -> ViewId {
        self.id
    }

    fn paint(&mut self, cx: &mut PaintCx) {
        // First draw the elevation colors as the base layer
        self.draw_elevation_colors(cx);
        // Then overlay the contour lines
        self.draw_contour_lines(cx);
    }

    fn view_style(&self) -> Option<Style> {
        // println!("view_style");
        Some(self.style.clone())
    }
}

pub fn create_topographic_map(heights: na::DMatrix<f32>) -> impl View {
    let config = TopographicConfig {
        width: 1200.0,
        height: 600.0,
        contour_interval: 25.0, // Draw contour every 25 units
        major_interval: 100.0,  // Major contours every 100 units
        ..Default::default()
    };

    let topo_map = TopographicMapView::new(heights, Some(config));

    // let view_id = test.id;

    // Create a lightweight handle for events
    // let handle = TimelineHandle {
    //     state: test.state.clone(),
    //     config: test.config.clone(),
    //     animation_data: test.animation_data.clone(),
    //     view_id,
    // };

    // let handle_move = handle.clone();
    // let handle_up = handle.clone();
    // let handle_wheel = handle.clone();

    container((topo_map))
        .style(|s| {
            s.width(1200.0)
                .height(600.0)
                .margin_top(50.0)
                .margin_left(50.0)
                .background(Color::LIGHT_CORAL)
        })
        .on_event(EventListener::PointerDown, move |e| {
            println!("PointerDown");
            let scale_factor = 1.25; // hardcode test
            let position = Point::new(
                e.point().expect("Couldn't get point").x as f64,
                e.point().expect("Couldn't get point").y as f64,
            );

            // handle_mouse_down(
            //     handle.state,
            //     handle.config.clone(),
            //     handle.animation_data.clone(),
            //     position,
            // );
            // handle.view_id.request_paint(); // Request repaint after state change
            EventPropagation::Continue
        })
        .on_event(EventListener::PointerMove, move |e| {
            // println!("PointerMove");
            let scale_factor = 1.25;
            let position = Point::new(
                e.point().expect("Couldn't get point").x as f64,
                e.point().expect("Couldn't get point").y as f64,
            );

            // handle_mouse_move(
            //     handle_move.state,
            //     handle_move.config.clone(),
            //     handle_move.animation_data.clone(),
            //     position,
            // );
            // handle.view_id.request_paint(); // Request repaint after state change
            EventPropagation::Continue
        })
        .on_event(EventListener::PointerUp, move |e| {
            println!("PointerUp");
            let scale_factor = 1.25;
            let position = Point::new(
                e.point().expect("Couldn't get point").x as f64,
                e.point().expect("Couldn't get point").y as f64,
            );
            // handle_mouse_up(handle_up.state, position);
            // handle.view_id.request_paint(); // Request repaint after state change
            EventPropagation::Continue
        })
        .on_event(EventListener::PointerWheel, move |e| {
            println!("PointerWheel");
            // Add wheel handling
            // handle_scroll(handle_wheel.state, 0.1);
            // handle.view_id.request_paint(); // Request repaint after state change
            EventPropagation::Continue
        })
}
