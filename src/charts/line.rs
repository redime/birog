use std::cmp::Ordering;
use std::fmt::Display;

use druid::piet::{FontBuilder, Text, TextLayout, TextLayoutBuilder};
use druid::widget::prelude::*;
use druid::{kurbo::BezPath, kurbo::Circle, theme, Affine, Color, Data, LinearGradient, Point, Rect, UnitPoint};
use num_traits::{AsPrimitive, Num};

use crate::charts::wilkinson;

#[derive(Clone, Debug)]
pub struct Line<X, Y> {
  points: Vec<(X, Y)>,
  color: Color,
}

#[derive(Clone, Debug)]
pub struct LineChartData<X, Y> {
  title: Option<String>,
  lines: Vec<Line<X, Y>>,
}

pub struct LineChart {
  cursor_pos: Point,
}

impl LineChart {
  pub fn new() -> Self {
    Self {
      cursor_pos: Point::new(-1.0, -1.0),
    }
  }
}

impl<X, Y> Line<X, Y> {
  pub fn new(points: Vec<(X, Y)>, color: Color) -> Self {
    Self { points, color }
  }
}

impl<X, Y> LineChartData<X, Y> {
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
  X: AsPrimitive<f64> + PartialEq,
  Y: AsPrimitive<f64> + PartialEq,
{
  fn same(&self, _other: &Self) -> bool {
    false
  }
}

impl<X, Y> Widget<LineChartData<X, Y>> for LineChart
where
  X: Display + AsPrimitive<f64> + PartialOrd + Num,
  Y: Display + AsPrimitive<f64> + PartialOrd + Num,
{
  fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut LineChartData<X, Y>, _env: &Env) {
    match event {
      Event::MouseMove(e) => {
        self.cursor_pos = e.pos;
        ctx.request_paint();
      }
      _ => {}
    }
  }

  fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &LineChartData<X, Y>, _env: &Env) {}

  fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &LineChartData<X, Y>, _data: &LineChartData<X, Y>, _env: &Env) {
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
    let size = ctx.size();

    let title_font_size = 25.0;
    let title_spacing = 40.0;
    let footer_spacing = 0.;

    let title_spacing = if let Some(ref title) = data.title {
      let font = ctx
        .text()
        .new_font_by_name("San Francisco", title_font_size)
        .build()
        .unwrap();
      let layout = ctx
        .text()
        .new_text_layout(&font, title, std::f64::INFINITY)
        .build()
        .unwrap();

      let width = size.width - layout.width();
      ctx.draw_text(
        &layout,
        (width / 2.0, (title_spacing + title_font_size) / 2.0),
        &env.get(theme::PRIMARY_LIGHT),
      );

      title_spacing
    } else {
      0.0
    };

    let padding = 50.;

    let label_tick_length = 5.;
    let label_font_name = "San Francisco";
    let label_font_size = 12.0;
    let path_stroke_width = 2.0;

    let min_label_spacing_w = label_font_size / 0.3;
    let min_label_spacing_h = label_font_size / 0.4;

    let bound_x = size.width - padding * 2.;
    let bound_y = size.height - padding * 2. - title_spacing - footer_spacing;

    let max_labels_x = (bound_x / min_label_spacing_w).floor().max(1.0);
    let max_labels_y = (bound_y / min_label_spacing_h).floor().max(1.0);

    let label_font = ctx
      .text()
      .new_font_by_name(label_font_name, label_font_size)
      .build()
      .unwrap();

    // region X Axis

    let min_x = data
      .lines
      .iter()
      .flat_map(|l| l.points.iter())
      .map(|(x, _)| x.as_())
      .min_by(|a, b| a.total_cmp(b))
      .unwrap_or(1.0f64);

    let max_x = data
      .lines
      .iter()
      .flat_map(|l| l.points.iter())
      .map(|(x, _)| x.as_())
      .max_by(|a, b| a.total_cmp(b))
      .unwrap_or(1.0f64);

    let labels_x = wilkinson::generate_labels(min_x, max_x, max_labels_x, wilkinson::LabelRange::Included);

    let precision_x = labels_x
      .iter()
      .map(|&x| get_precision(x))
      .max_by(|a, b| a.total_cmp(b))
      .unwrap_or(1.0);

    let proportion_x = bound_x / (max_x - min_x).abs();

    // endregion

    // region Y Axis

    let min_y = data
      .lines
      .iter()
      .flat_map(|l| l.points.iter())
      .map(|(_, y)| y.as_())
      .min_by(|a, b| a.total_cmp(b))
      .unwrap_or(1.0f64);

    let max_y = data
      .lines
      .iter()
      .flat_map(|l| l.points.iter())
      .map(|(_, y)| y.as_())
      .max_by(|a, b| a.total_cmp(b))
      .unwrap_or(1.0f64);

    let min_y = min_y - (max_y - min_y) * 0.1;
    let max_y = max_y + (max_y - min_y) * 0.1;

    let labels_y = wilkinson::generate_labels(min_y, max_y, max_labels_y, wilkinson::LabelRange::Included);

    let precision_y = labels_y
      .iter()
      .map(|&y| get_precision(y))
      .max_by(|a, b| a.total_cmp(b))
      .unwrap_or(1.0);

    let precision_values_y = data
      .lines
      .iter()
      .flat_map(|l| l.points.iter())
      .map(|(_, y)| get_precision(y.as_()))
      .max_by(|a, b| a.total_cmp(b))
      .unwrap_or(1.0);

    let proportion_y = bound_y / (max_y - min_y);

    // endregion

    // region Chart Grid and Labels

    // Draw a square around the chart
    let rect = Rect::from_points(
      Point::new(padding, padding + title_spacing),
      Point::new(size.width - padding, size.height - padding - footer_spacing),
    );

    ctx.stroke(rect, &env.get(theme::PRIMARY_LIGHT), 1.0);

    for value_x in labels_x.iter().skip_while(|&&v| v < min_x).take_while(|&&v| v <= max_x) {
      let label = format!("{:.prec$}", value_x, prec = (precision_x as usize));

      let layout = ctx
        .text()
        .new_text_layout(&label_font, &label, std::f64::INFINITY)
        .build()
        .unwrap();

      let pos_x = (value_x - min_x) * proportion_x + padding;
      let pos_y_top = padding + title_spacing;
      let pos_y_bottom = size.height - padding - footer_spacing;

      // Top label
      ctx.draw_text(
        &layout,
        (pos_x - layout.width() / 2.0, pos_y_top - label_font_size),
        &env.get(theme::PRIMARY_LIGHT),
      );

      // Bottom label
      ctx.draw_text(
        &layout,
        (
          pos_x - layout.width() / 2.0,
          pos_y_bottom + label_font_size + label_tick_length * 2.0,
        ),
        &env.get(theme::PRIMARY_LIGHT),
      );

      // Ticks
      let mut tick_line = BezPath::new();
      tick_line.move_to((pos_x, pos_y_top));
      tick_line.line_to((pos_x, pos_y_top - label_tick_length));

      tick_line.move_to((pos_x, pos_y_bottom));
      tick_line.line_to((pos_x, pos_y_bottom + label_tick_length));

      ctx.stroke(tick_line, &env.get(theme::PRIMARY_LIGHT), 1.0);

      // Grid line
      let mut grid_line = BezPath::new();
      grid_line.move_to((pos_x, pos_y_top));
      grid_line.line_to((pos_x, pos_y_bottom));

      ctx.stroke(grid_line, &env.get(theme::PRIMARY_LIGHT).with_alpha(0.1), 1.0);
    }

    for value_y in labels_y.iter().skip_while(|&&v| v < min_y).take_while(|&&v| v <= max_y) {
      let label = format!("{:.prec$}", value_y, prec = (precision_y as usize));

      let layout = ctx
        .text()
        .new_text_layout(&label_font, &label, std::f64::INFINITY)
        .build()
        .unwrap();

      let pos_y = size.height - (value_y - min_y) * proportion_y - padding - footer_spacing;

      // Left label
      ctx.draw_text(
        &layout,
        (
          padding - layout.width() - label_tick_length * 2.0,
          pos_y + label_font_size / 2.5,
        ),
        &env.get(theme::PRIMARY_LIGHT),
      );

      // Right label
      ctx.draw_text(
        &layout,
        (
          size.width - padding + label_tick_length * 2.0,
          pos_y + label_font_size / 2.5,
        ),
        &env.get(theme::PRIMARY_LIGHT),
      );

      // Ticks
      let mut tick_line = BezPath::new();
      tick_line.move_to((padding, pos_y));
      tick_line.line_to((padding - label_tick_length, pos_y));

      tick_line.move_to((size.width - padding, pos_y));
      tick_line.line_to((size.width - (padding - label_tick_length), pos_y));

      ctx.stroke(tick_line, &env.get(theme::PRIMARY_LIGHT), 1.0);

      // Grid line
      let mut grid_line = BezPath::new();
      grid_line.move_to((padding, pos_y));
      grid_line.line_to((size.width - padding, pos_y));

      ctx.stroke(grid_line, &env.get(theme::PRIMARY_LIGHT).with_alpha(0.1), 1.0);
    }

    // endregion

    // region Chart lines

    for line in data.lines.iter() {
      let mut line_path = BezPath::new();
      let mut line_polygon = BezPath::new();

      // Move first point into position
      if let Some((first_x, first_y)) = line.points.first() {
        let pos_x = (first_x.as_() - min_x) * proportion_x + padding;
        let pos_y = size.height - (first_y.as_() - min_y) * proportion_y - padding - footer_spacing;

        line_path.move_to((pos_x, pos_y));
        line_polygon.move_to((pos_x, size.height - padding - footer_spacing));
        line_polygon.line_to((pos_x, pos_y));
      }

      // Draw the path along the chart area
      for (x, y) in line.points.iter().skip(1) {
        let pos_x = (x.as_() - min_x) * proportion_x + padding;
        let pos_y = size.height - (y.as_() - min_y) * proportion_y - padding - footer_spacing;

        line_path.line_to((pos_x, pos_y));
        line_polygon.line_to((pos_x, pos_y));
      }

      ctx.stroke(line_path.clone(), &line.color, path_stroke_width);

      if let Some((last_x, _)) = line.points.iter().last() {
        let pos_x = (last_x.as_() - min_x) * proportion_x + padding;

        line_polygon.line_to((pos_x, size.height - padding - footer_spacing));
        ctx.fill(
          line_polygon.clone(),
          &LinearGradient::new(
            UnitPoint::TOP,
            UnitPoint::BOTTOM,
            (line.color.clone().with_alpha(0.5), line.color.clone().with_alpha(0.0)),
          ),
        );
      }

      // Draw a circle in the closest point to the cursor position
      if self.cursor_pos.x > padding
        && self.cursor_pos.x < size.width - padding
        && self.cursor_pos.y > padding + title_spacing
        && self.cursor_pos.y < size.height - padding - footer_spacing
      {
        let comparison = line.points.iter().min_by(|(a, _), (b, _)| {
          ((a.as_() - min_x) - (self.cursor_pos.x - padding) / proportion_x)
            .abs()
            .total_cmp(&((b.as_() - min_x) - (self.cursor_pos.x - padding) / proportion_x).abs())
        });

        if let Some((x, y)) = comparison {
          let pos_x = (-min_x + x.as_()) * proportion_x + padding;
          let pos_y = size.height - (-min_y + y.as_()) * proportion_y - padding - footer_spacing;

          // Add circle emphasizing the point
          let path = Circle::new((pos_x, pos_y), 4.0);
          ctx.fill(path.clone(), &line.color);
          ctx.stroke(path, &env.get(theme::PRIMARY_DARK), path_stroke_width);

          let layout = ctx
            .text()
            .new_text_layout(
              &label_font,
              &format!("{:.prec$}", y, prec = precision_values_y as usize),
              std::f64::INFINITY,
            )
            .build()
            .unwrap();

          let spacing = 5.0;

          if pos_x + layout.width() < size.width - padding - 15.0 {
            let rect = Rect::from_points(
              Point::new(pos_x + 8.0, pos_y - 5.0 - (label_font_size / 2.0)),
              Point::new(pos_x + 18.0 + layout.width(), pos_y + 5.0 + (label_font_size / 2.0)),
            );

            ctx.fill(rect, &env.get(theme::PRIMARY_LIGHT));

            ctx.draw_text(
              &layout,
              (pos_x + 13.0, pos_y + (label_font_size * 0.334)),
              &env.get(theme::PRIMARY_DARK),
            );
          } else {
            let rect = Rect::from_points(
              Point::new(pos_x - 8.0, pos_y - (label_font_size / 2.0)),
              Point::new(pos_x - 18.0 - layout.width(), pos_y + (label_font_size / 2.0)),
            );

            ctx.fill(rect, &env.get(theme::PRIMARY_LIGHT));

            ctx.draw_text(
              &layout,
              (pos_x - 13.0 - layout.width(), pos_y + (label_font_size * 0.334)),
              &env.get(theme::PRIMARY_DARK),
            );
          }
        }
      }
    }

    // endregion

    // region Cursor reference lines

    if self.cursor_pos.x > padding
      && self.cursor_pos.x < size.width - padding
      && self.cursor_pos.y > padding + title_spacing
      && self.cursor_pos.y < size.height - padding - footer_spacing
    {
      // Y
      let mut line_path = BezPath::new();

      let mut x = padding;
      while x <= size.width - padding {
        line_path.move_to((x.min(size.width - padding), self.cursor_pos.y));
        line_path.line_to(((x + 5.0).min(size.width - padding), self.cursor_pos.y));

        x += 10.0;
      }

      ctx.stroke(line_path, &env.get(theme::PRIMARY_LIGHT).with_alpha(0.3), 1.0);

      // Draw reference value at the end
      let value = ((size.height - footer_spacing - padding) - self.cursor_pos.y) / proportion_y + min_y;

      let layout = ctx
        .text()
        .new_text_layout(
          &label_font,
          &format!("{:.prec$}", value, prec = precision_values_y as usize),
          std::f64::INFINITY,
        )
        .build()
        .unwrap();

      let origin_hrz = size.width - padding;
      let rect = Rect::from_points(
        Point::new(origin_hrz, self.cursor_pos.y - (10.0 + label_font_size) / 2.0),
        // Discover max width possible?
        Point::new(
          origin_hrz + layout.width() + 10.0,
          self.cursor_pos.y + (10.0 + label_font_size) / 2.0,
        ),
      );

      ctx.fill(rect, &env.get(theme::PRIMARY_LIGHT));

      ctx.draw_text(
        &layout,
        (origin_hrz + 5.0, self.cursor_pos.y + (label_font_size * 0.334)),
        &env.get(theme::PRIMARY_DARK),
      );

      // X

      let mut line_path = BezPath::new();

      let pos_y_top = padding + title_spacing;
      let pos_y_bottom = size.height - padding - footer_spacing;

      let mut y = pos_y_top;
      while y <= size.width - padding {
        line_path.move_to((self.cursor_pos.x, y.min(pos_y_bottom)));
        line_path.line_to((self.cursor_pos.x, (y + 5.0).min(pos_y_bottom)));

        y += 10.0;
      }

      ctx.stroke(line_path, &env.get(theme::PRIMARY_LIGHT).with_alpha(0.3), 1.0);

      // Draw reference value at the end
      let value = (self.cursor_pos.x - padding) / proportion_x + min_x;

      let layout = ctx
        .text()
        .new_text_layout(
          &label_font,
          &format!("{:.prec$}", value, prec = precision_x as usize),
          std::f64::INFINITY,
        )
        .build()
        .unwrap();

      let rect = Rect::from_points(
        Point::new(self.cursor_pos.x - layout.width() / 2.0 - 5.0, pos_y_bottom),
        Point::new(self.cursor_pos.x + layout.width() / 2.0 + 5.0, pos_y_bottom + 25.0),
      );

      ctx.fill(rect, &env.get(theme::PRIMARY_LIGHT));

      ctx.draw_text(
        &layout,
        (self.cursor_pos.x - layout.width() / 2.0, pos_y_bottom + 17.0),
        &env.get(theme::PRIMARY_DARK),
      );
    }

    // endregion
  }
}

fn get_precision(i: f64) -> f64 {
  let mut e = 1.0f64;
  while (i * e).round() / e != i {
    e *= 10.;
  }

  return (e.ln() / 10.0f64.ln()).round();
}
