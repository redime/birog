// Copyright 2020 The Birog Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cmp::Ordering;
use std::fmt::Display;

use druid::piet::{FontBuilder, Text, TextLayout, TextLayoutBuilder};
use druid::widget::prelude::*;
use druid::{
  kurbo::BezPath, kurbo::Circle, theme, Color, Data, LinearGradient, Point, Rect, UnitPoint,
};
use num_traits::{AsPrimitive, Num};

use crate::charts::wilkinson;

#[derive(Clone, Debug)]
pub struct Line<X, Y> {
  points: Vec<(X, Y)>,
  color: Color,
}

#[derive(Clone, Debug)]
pub struct LineChartData<X, Y>
where
  X: Data,
  Y: Data,
{
  title: Option<String>,
  lines: Vec<Line<X, Y>>,
}

pub struct LineChart {
  cursor_pos: Point,
  settings: LineChartSettings,
  min_x: f64,
  max_x: f64,
  min_y: f64,
  max_y: f64,
  precision_x: usize,
  precision_y: usize,
  proportion_x: f64,
  proportion_y: f64,
}

struct LineChartSettings {
  font_size: f64,
  padding_top: f64,
  padding_bottom: f64,
  padding_left: f64,
  padding_right: f64,
  header_height: f64,
  footer_height: f64,
  tick_length: f64,
  path_stroke_width: f64,
}

impl LineChart {
  pub fn new() -> Self {
    Self {
      cursor_pos: Point::new(-1.0, -1.0),
      settings: LineChartSettings {
        font_size: 12.0,
        padding_top: 50.0,
        padding_bottom: 50.0,
        padding_left: 50.0,
        padding_right: 50.0,
        header_height: 0.0,
        footer_height: 0.0,
        tick_length: 5.0,
        path_stroke_width: 2.0,
      },
      min_x: 0.0,
      max_x: 0.0,
      min_y: 0.0,
      max_y: 0.0,
      precision_x: 0,
      precision_y: 0,
      proportion_x: 0.0,
      proportion_y: 0.0,
    }
  }

  fn update_reference_data<X, Y>(&mut self, data: &LineChartData<X, Y>)
  where
    X: Num + Data + AsPrimitive<f64>,
    Y: Num + Data + AsPrimitive<f64>,
  {
    let x_iter = data
      .lines
      .iter()
      .flat_map(|l| l.points.iter())
      .map(|(x, _)| x.as_());

    self.min_x = x_iter
      .clone()
      .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
      .unwrap_or(1.0);

    self.max_x = x_iter
      .clone()
      .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
      .unwrap_or(1.0);

    self.precision_x = x_iter
      .clone()
      .map(|x| get_precision(x))
      .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
      .unwrap_or(1);

    let y_iter = data
      .lines
      .iter()
      .flat_map(|l| l.points.iter())
      .map(|(_, y)| y.as_());

    self.min_y = y_iter
      .clone()
      .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
      .unwrap_or(1.0)
      * 0.95;

    self.max_y = y_iter
      .clone()
      .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
      .unwrap_or(1.0)
      * 1.05;

    self.precision_y = y_iter
      .clone()
      .map(|y| get_precision(y))
      .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
      .unwrap_or(1);
  }

  fn get_axis(&self, min_value: f64, max_value: f64, max_labels: f64) -> (Vec<f64>, usize) {
    let labels = wilkinson::generate_labels(
      min_value,
      max_value,
      max_labels,
      wilkinson::LabelRange::Included,
    );

    let precision = labels
      .iter()
      .map(|v| get_precision(*v))
      .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
      .unwrap_or(1);

    (labels, precision)
  }

  fn paint_labels(&mut self, ctx: &mut PaintCtx, env: &Env) {
    let size = ctx.size();

    let label_font = ctx
      .text()
      .new_font_by_name(&env.get(theme::FONT_NAME), self.settings.font_size)
      .build()
      .unwrap();

    let min_label_spacing_h = self.settings.font_size / 0.3;
    let min_label_spacing_v = self.settings.font_size / 0.4;

    let padding_h = self.settings.padding_left + self.settings.padding_right;
    let padding_v = self.settings.padding_top + self.settings.padding_bottom;

    let bounds_h = size.width - padding_h;
    let bounds_v =
      size.height - padding_v - self.settings.header_height - self.settings.footer_height;

    let max_labels_x = (bounds_h / min_label_spacing_h).floor().max(1.0);
    let max_labels_y = (bounds_v / min_label_spacing_v).floor().max(1.0);

    let (x_axis, x_axis_precision) = self.get_axis(self.min_x, self.max_x, max_labels_x);
    let (y_axis, y_axis_precision) = self.get_axis(self.min_y, self.max_y, max_labels_y);

    let origin_left = self.settings.padding_left;
    let origin_right = size.width - self.settings.padding_right;
    let origin_top = self.settings.padding_top + self.settings.header_height;
    let origin_bottom = size.height - self.settings.footer_height - self.settings.padding_bottom;

    self.min_x = self.min_x;
    self.min_y = self.min_y.min(y_axis[0]);

    self.max_x = self.max_x;
    self.max_y = self.max_y.max(y_axis[y_axis.len() - 1]);

    self.proportion_x = bounds_h / (self.max_x - self.min_x).abs();
    self.proportion_y = bounds_v / (self.max_y - self.min_y).abs();

    // Draw chart rectangle
    let rect = Rect::from_points(
      Point::new(origin_left, origin_top),
      Point::new(origin_right, origin_bottom),
    );

    ctx.stroke(rect, &env.get(theme::FOREGROUND_DARK), 1.0);

    for value_x in x_axis
      .iter()
      .skip_while(|v| **v < self.min_x.as_())
      .take_while(|v| **v <= self.max_x.as_())
    {
      let label = format!("{:.prec$}", value_x, prec = x_axis_precision);

      let layout = ctx
        .text()
        .new_text_layout(&label_font, &label, std::f64::INFINITY)
        .build()
        .unwrap();

      let position_x = origin_left + (value_x - self.min_x) * self.proportion_x;

      ctx.draw_text(
        &layout,
        (
          position_x - layout.width() / 2.0,
          origin_top - self.settings.tick_length - 2.0,
        ),
        &env.get(theme::FOREGROUND_DARK),
      );

      ctx.draw_text(
        &layout,
        (
          position_x - layout.width() / 2.0,
          origin_bottom + self.settings.tick_length + self.settings.font_size,
        ),
        &env.get(theme::FOREGROUND_DARK),
      );

      // Ticks
      let mut tick_line = BezPath::new();
      tick_line.move_to((position_x, origin_top));
      tick_line.line_to((position_x, origin_top - self.settings.tick_length));

      tick_line.move_to((position_x, origin_bottom));
      tick_line.line_to((position_x, origin_bottom + self.settings.tick_length));

      ctx.stroke(tick_line, &env.get(theme::FOREGROUND_DARK), 1.0);

      // Grid line
      let mut grid_line = BezPath::new();
      grid_line.move_to((position_x, origin_top));
      grid_line.line_to((position_x, origin_bottom));

      ctx.stroke(
        grid_line,
        &env.get(theme::FOREGROUND_DARK).with_alpha(0.1),
        1.0,
      );
    }

    for value_y in y_axis
      .iter()
      .skip_while(|v| **v < self.min_y.as_())
      .take_while(|v| **v <= self.max_y.as_())
    {
      let label = format!("{:.prec$}", value_y, prec = y_axis_precision);

      let layout = ctx
        .text()
        .new_text_layout(&label_font, &label, std::f64::INFINITY)
        .build()
        .unwrap();

      let position_y = origin_bottom - (value_y - self.min_y) * self.proportion_y;
      let text_height_adjustment = if let Some(metric) = layout.line_metric(0) {
        metric.cumulative_height - metric.baseline.floor()
      } else {
        self.settings.font_size / 2.2
      };

      ctx.draw_text(
        &layout,
        (
          origin_left - layout.width() - self.settings.tick_length - 2.0,
          position_y + text_height_adjustment,
        ),
        &env.get(theme::FOREGROUND_DARK),
      );

      ctx.draw_text(
        &layout,
        (
          origin_right + self.settings.tick_length + 2.0,
          position_y + text_height_adjustment,
        ),
        &env.get(theme::FOREGROUND_DARK),
      );

      // Ticks
      let mut tick_line = BezPath::new();
      tick_line.move_to((origin_left, position_y));
      tick_line.line_to((origin_left - self.settings.tick_length, position_y));

      tick_line.move_to((origin_right, position_y));
      tick_line.line_to((origin_right + self.settings.tick_length, position_y));

      ctx.stroke(tick_line, &env.get(theme::FOREGROUND_DARK), 1.0);

      // Grid line
      let mut grid_line = BezPath::new();
      grid_line.move_to((origin_left, position_y));
      grid_line.line_to((origin_right, position_y));

      ctx.stroke(
        grid_line,
        &env.get(theme::FOREGROUND_DARK).with_alpha(0.1),
        1.0,
      );
    }
  }

  fn paint_lines<X, Y>(&self, ctx: &mut PaintCtx, lines: &[Line<X, Y>], env: &Env)
  where
    X: Num + AsPrimitive<f64>,
    Y: Num + AsPrimitive<f64> + Display,
  {
    let size = ctx.size();

    let origin_left = self.settings.padding_left;
    let origin_right = size.width - self.settings.padding_right;
    let origin_top = self.settings.padding_top + self.settings.header_height;
    let origin_bottom = size.height - self.settings.footer_height - self.settings.padding_bottom;

    let label_font = ctx
      .text()
      .new_font_by_name(&env.get(theme::FONT_NAME), self.settings.font_size)
      .build()
      .unwrap();

    for line in lines.iter() {
      let mut line_path = BezPath::new();
      let mut line_polygon = BezPath::new();

      // Move first point into position
      if let Some((first_x, first_y)) = line.points.first() {
        let pos_x = origin_left + (first_x.as_() - self.min_x) * self.proportion_x;
        let pos_y = origin_bottom - (first_y.as_() - self.min_y) * self.proportion_y;

        line_path.move_to((pos_x, pos_y));
        line_polygon.move_to((pos_x, origin_bottom));
        line_polygon.line_to((pos_x, pos_y));
      }

      // Draw the path along the chart area
      for (x, y) in line.points.iter().skip(1) {
        let pos_x = origin_left + (x.as_() - self.min_x) * self.proportion_x;
        let pos_y = origin_bottom - (y.as_() - self.min_y) * self.proportion_y;

        line_path.line_to((pos_x, pos_y));
        line_polygon.line_to((pos_x, pos_y));
      }

      ctx.stroke(
        line_path.clone(),
        &line.color,
        self.settings.path_stroke_width,
      );

      if let Some((last_x, _)) = line.points.iter().last() {
        let pos_x = origin_left + (last_x.as_() - self.min_x) * self.proportion_x;

        line_polygon.line_to((pos_x, origin_bottom));
        ctx.fill(
          line_polygon.clone(),
          &LinearGradient::new(
            UnitPoint::TOP,
            UnitPoint::BOTTOM,
            (
              line.color.clone().with_alpha(0.5),
              line.color.clone().with_alpha(0.0),
            ),
          ),
        );
      }

      // Highlight the closest point to the cursor position
      if self.cursor_pos.x > origin_left
        && self.cursor_pos.x < origin_right
        && self.cursor_pos.y > origin_top
        && self.cursor_pos.y < origin_bottom
      {
        let closest_point = line.points.iter().min_by(|(a, _), (b, _)| {
          ((a.as_() - self.min_x) - (self.cursor_pos.x - origin_left) / self.proportion_x)
            .abs()
            .partial_cmp(
              &((b.as_() - self.min_x) - (self.cursor_pos.x - origin_left) / self.proportion_x)
                .abs(),
            )
            .unwrap_or(Ordering::Equal)
        });

        if let Some((x, y)) = closest_point {
          let pos_x = origin_left + (x.as_() - self.min_x) * self.proportion_x;
          let pos_y = origin_bottom - (y.as_() - self.min_y) * self.proportion_y;

          // Add circle emphasizing the point
          let path = Circle::new((pos_x, pos_y), 4.0);
          ctx.fill(path.clone(), &line.color);
          ctx.stroke(
            path,
            &env.get(theme::BACKGROUND_DARK),
            self.settings.path_stroke_width,
          );

          let layout = ctx
            .text()
            .new_text_layout(
              &label_font,
              &format!("{:.prec$}", y, prec = self.precision_y),
              std::f64::INFINITY,
            )
            .build()
            .unwrap();

          let text_height = if let Some(metric) = layout.line_metric(0) {
            self.settings.font_size - (metric.cumulative_height - metric.baseline.floor())
          } else {
            self.settings.font_size
          };

          // Draw box with the point Y value
          if pos_x + layout.width() < origin_right - 15.0 {
            let rect = Rect::from_points(
              Point::new(pos_x + 8.0, pos_y - 5.0 - text_height / 2.0),
              Point::new(
                pos_x + 18.0 + layout.width(),
                pos_y + 5.0 + text_height / 2.0,
              ),
            );

            ctx.fill(rect, &env.get(theme::FOREGROUND_DARK));

            ctx.draw_text(
              &layout,
              (pos_x + 13.0, pos_y + (self.settings.font_size * 0.334)),
              &env.get(theme::BACKGROUND_DARK),
            );
          } else {
            let rect = Rect::from_points(
              Point::new(pos_x - 8.0, pos_y - 5.0 - text_height / 2.0),
              Point::new(
                pos_x - 18.0 - layout.width(),
                pos_y + 5.0 + text_height / 2.0,
              ),
            );

            ctx.fill(rect, &env.get(theme::FOREGROUND_DARK));

            ctx.draw_text(
              &layout,
              (
                pos_x - 13.0 - layout.width(),
                pos_y + (self.settings.font_size * 0.334),
              ),
              &env.get(theme::BACKGROUND_DARK),
            );
          }
        }
      }
    }
  }

  fn paint_cursor_reference(&self, ctx: &mut PaintCtx, env: &Env) {
    let size = ctx.size();

    let origin_left = self.settings.padding_left;
    let origin_right = size.width - self.settings.padding_right;
    let origin_top = self.settings.padding_top + self.settings.header_height;
    let origin_bottom = size.height - self.settings.footer_height - self.settings.padding_bottom;

    if self.cursor_pos.x > origin_left
      && self.cursor_pos.x < origin_right
      && self.cursor_pos.y > origin_top
      && self.cursor_pos.y < origin_bottom
    {
      // Y
      let mut line_path = BezPath::new();

      let mut x = origin_left;
      while x <= origin_right {
        line_path.move_to((x.min(origin_right), self.cursor_pos.y));
        line_path.line_to(((x + 5.0).min(origin_right), self.cursor_pos.y));

        x += 10.0;
      }

      ctx.stroke(
        line_path,
        &env.get(theme::FOREGROUND_DARK).with_alpha(0.3),
        1.0,
      );

      // Draw reference value at the end
      let value = (origin_bottom - self.cursor_pos.y) / self.proportion_y + self.min_y;

      let label_font = ctx
        .text()
        .new_font_by_name(&env.get(theme::FONT_NAME), self.settings.font_size)
        .build()
        .unwrap();

      let layout = ctx
        .text()
        .new_text_layout(
          &label_font,
          &format!("{:.prec$}", value, prec = self.precision_y as usize),
          std::f64::INFINITY,
        )
        .build()
        .unwrap();

      let text_height = if let Some(metric) = layout.line_metric(0) {
        self.settings.font_size - (metric.cumulative_height - metric.baseline.floor())
      } else {
        self.settings.font_size
      };

      let rect = Rect::from_points(
        Point::new(origin_right, self.cursor_pos.y - (10.0 + text_height) / 2.0),
        // Discover max width possible?
        Point::new(
          origin_right + layout.width() + 10.0,
          self.cursor_pos.y + (10.0 + text_height) / 2.0,
        ),
      );

      ctx.fill(rect, &env.get(theme::FOREGROUND_DARK));

      ctx.draw_text(
        &layout,
        (
          origin_right + 5.0,
          self.cursor_pos.y + (self.settings.font_size * 0.334),
        ),
        &env.get(theme::BACKGROUND_DARK),
      );

      // X

      let mut line_path = BezPath::new();

      let mut y = origin_left;
      while y <= origin_right {
        line_path.move_to((self.cursor_pos.x, y.min(origin_bottom)));
        line_path.line_to((self.cursor_pos.x, (y + 5.0).min(origin_bottom)));

        y += 10.0;
      }

      ctx.stroke(
        line_path,
        &env.get(theme::FOREGROUND_DARK).with_alpha(0.3),
        1.0,
      );

      // Draw reference value at the end
      let value = (self.cursor_pos.x - origin_left) / self.proportion_x + self.min_x;

      let layout = ctx
        .text()
        .new_text_layout(
          &label_font,
          &format!("{:.prec$}", value, prec = self.precision_x as usize),
          std::f64::INFINITY,
        )
        .build()
        .unwrap();

      let rect = Rect::from_points(
        Point::new(
          self.cursor_pos.x - layout.width() / 2.0 - 5.0,
          origin_bottom,
        ),
        Point::new(
          self.cursor_pos.x + layout.width() / 2.0 + 5.0,
          origin_bottom + text_height + 10.0,
        ),
      );

      ctx.fill(rect, &env.get(theme::FOREGROUND_DARK));

      ctx.draw_text(
        &layout,
        (
          self.cursor_pos.x - layout.width() / 2.0,
          origin_bottom + text_height + 3.0,
        ),
        &env.get(theme::BACKGROUND_DARK),
      );
    }
  }
}

impl<X, Y> Line<X, Y> {
  pub fn new(points: Vec<(X, Y)>, color: Color) -> Self {
    Self { points, color }
  }
}

impl<X, Y> LineChartData<X, Y>
where
  X: Display + Data + AsPrimitive<f64> + PartialOrd + Num,
  Y: Display + Data + AsPrimitive<f64> + PartialOrd + Num,
{
  pub fn new() -> Self {
    Self {
      title: None,
      lines: Vec::new(),
    }
  }

  pub fn with_title(mut self, title: impl Into<String>) -> Self {
    self.title = Some(title.into());
    self
  }

  pub fn with_line(mut self, line: Line<X, Y>) -> Self {
    self.lines.push(line);
    self
  }
}

impl<X, Y> Data for LineChartData<X, Y>
where
  X: AsPrimitive<f64> + Data + PartialEq,
  Y: AsPrimitive<f64> + Data + PartialEq,
{
  fn same(&self, other: &Self) -> bool {
    let basic_test = self.title == other.title && self.lines.len() == other.lines.len();

    basic_test
      && self
        .lines
        .iter()
        .zip(other.lines.iter())
        .all(|(line_a, line_b)| {
          line_a
            .points
            .iter()
            .zip(line_b.points.iter())
            .all(|(point_a, point_b)| point_a == point_b)
        })
  }
}

impl<X, Y> Widget<LineChartData<X, Y>> for LineChart
where
  X: Display + Data + AsPrimitive<f64> + PartialOrd + Num,
  Y: Display + Data + AsPrimitive<f64> + PartialOrd + Num,
{
  fn event(
    &mut self,
    ctx: &mut EventCtx,
    event: &Event,
    _data: &mut LineChartData<X, Y>,
    _env: &Env,
  ) {
    match event {
      Event::MouseMove(e) => {
        self.cursor_pos = e.pos;
        ctx.request_paint();
      }
      _ => {}
    }
  }

  fn lifecycle(
    &mut self,
    _ctx: &mut LifeCycleCtx,
    event: &LifeCycle,
    data: &LineChartData<X, Y>,
    _env: &Env,
  ) {
    match event {
      LifeCycle::WidgetAdded => self.update_reference_data(data),
      _ => (),
    }
  }

  fn update(
    &mut self,
    _ctx: &mut UpdateCtx,
    _old_data: &LineChartData<X, Y>,
    data: &LineChartData<X, Y>,
    _env: &Env,
  ) {
  }

  fn layout(
    &mut self,
    _layout_ctx: &mut LayoutCtx,
    bc: &BoxConstraints,
    _data: &LineChartData<X, Y>,
    _env: &Env,
  ) -> Size {
    bc.max()
  }

  fn paint(&mut self, ctx: &mut PaintCtx, data: &LineChartData<X, Y>, env: &Env) {
    self.paint_labels(ctx, env);
    self.paint_cursor_reference(ctx, env);
    self.paint_lines(ctx, &data.lines, env);
  }
}

fn get_precision<N>(i: N) -> usize
where
  N: Num + AsPrimitive<f64>,
{
  let i = i.as_();
  let mut e = 1.0f64;
  while (i * e).round() / e != i {
    e *= 10.;
  }

  return (e.ln() / 10.0f64.ln()).round() as usize;
}
