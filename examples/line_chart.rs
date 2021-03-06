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

use druid::widget::{Flex, Label, List, MainAxisAlignment, SizedBox};
use druid::{
  theme, AppLauncher, Color, Data, Lens, LocalizedString, Widget, WidgetExt, WindowDesc,
};

use birog::charts::line::{Line, LineChart, LineChartData};
use std::sync::Arc;

#[derive(Clone, Data, Lens)]
struct AppData {
  chart_data: LineChartData<i32, f64>,
  left: Arc<Vec<String>>,
}

fn main() {
  let window = WindowDesc::new(ui_builder)
    .window_size((1024., 500.))
    .title(LocalizedString::new("custom-widget-demo-window-title").with_placeholder("Chart Data"));

  AppLauncher::with_window(window)
    .configure_env(|env, _| {
      env.set(
        theme::WINDOW_BACKGROUND_COLOR,
        Color::rgb8(0x1F, 0x24, 0x30),
      );
      env.set(theme::FOREGROUND_DARK, Color::rgb8(0xCB, 0xCC, 0xC6));
    })
    .use_simple_logger()
    .launch(data_builder())
    .expect("launch failed");
}

fn ui_builder() -> impl Widget<AppData> {
  Flex::row()
    .must_fill_main_axis(true)
    .main_axis_alignment(MainAxisAlignment::SpaceBetween)
    .with_child(
      SizedBox::new(List::new(|| Label::new(|t: &String, _env: &_| t.into())).lens(AppData::left))
        .width(100.)
        .expand_height(),
    )
    .with_flex_child(
      SizedBox::new(LineChart::new().lens(AppData::chart_data))
        .expand_height()
        .expand_width(),
      1.0
    )
    .debug_paint_layout()
}

fn data_builder() -> AppData {
  let series_a = vec![
    34.14, 34.15, 33.89, 33.23, 33.43, 32.72, 32.44, 32.26, 31.35, 31.23, 30.59, 30.45, 29.89,
    29.28, 30.08, 30.49, 30.87, 30.54, 30.94, 31.57, 30.72, 31.2, 31.4, 30.91, 31.03, 31.24, 31.07,
    31.03, 31.07, 30.56, 30.65, 31.04, 30.8, 30.94, 31.28, 31.22, 31.27, 31.29, 30.65, 30.79,
    29.37, 29.73, 28.98, 28.95, 29.51, 29.31, 28.73, 29.11, 29.7, 29.63, 29.22, 29.45, 30.04,
    30.68, 30.67, 30.22, 30.36, 30.21, 31.18, 30.7, 30.94, 30.74, 30.95, 30.76, 31.01, 31.1, 30.88,
    30.76, 31.3, 30.61, 30.43, 30.39, 30.18, 30.18, 29.92, 30.74, 30.91, 30.64, 30.32, 30.42,
    30.71, 31.41, 32.12, 31.72, 32.01, 31.18, 31.03, 31.5, 31.97, 32.31, 33.19, 32.74, 32.06,
    31.43, 31.48, 31.34, 32.19, 32.68, 33.08, 33.63, 33.38, 33.7, 33.6, 33.71, 32.53, 32.15, 32.31,
    32.47, 32.92, 32.74, 33.04, 32.24, 31.97, 31.61, 31.48, 31.26, 31.56, 31.02, 30.68, 30.79,
    30.98, 30.88, 30.8, 30.93, 30.63, 31.37, 30.67, 30.68, 30.65, 30.81, 30.56, 31.45, 31.64,
    30.76, 30.27, 30.46, 30.02, 30.09, 30.23, 30.02, 29.2, 28.62, 28.31, 28.25, 28.53, 28.22,
    27.69, 27.44, 27.35, 28.13, 28.62, 27.68, 27.66, 27.86, 28.16, 28.42, 29.01, 29.25, 29.54,
    29.36, 28.59, 28.45, 27.66, 27.71, 28.36, 28.82, 28.98, 28.07, 28.15, 27.92, 27.93, 27.93,
    27.9, 28.23, 27.86, 27.29, 27.39, 27.36, 27.01, 26.86, 26.37, 26.64, 26.0, 25.82, 25.82, 26.3,
    25.97, 26.27, 27.45, 27.7, 27.35, 26.81, 26.68, 26.57, 27.5, 28.24, 27.92, 28.27, 28.44, 28.29,
    28.9, 28.98, 29.05, 30.03, 30.09, 30.28, 30.0, 31.08, 30.92, 31.18, 30.84, 31.61, 31.51, 31.64,
    31.74, 32.01, 31.59, 31.1, 30.84, 30.42, 30.71, 30.2, 29.75, 30.33, 30.62, 30.62, 30.3, 29.44,
    29.48, 29.08, 29.42, 28.68, 28.66, 29.63, 29.46, 29.25, 29.12, 29.21, 28.59, 29.6, 30.34, 29.6,
    30.26, 30.82, 30.71, 30.26, 29.56, 29.57, 29.58, 30.45, 30.56, 31.0, 31.46, 30.99, 30.92,
    30.59, 30.85, 31.23, 31.58, 31.31, 31.49, 31.1, 31.07, 31.55, 31.6, 32.08, 32.25, 31.33, 31.13,
    30.62, 30.31, 30.43, 30.31, 29.96, 29.84, 30.4, 30.34, 29.38, 29.6, 30.13, 30.72, 30.4, 29.06,
    29.19, 29.27, 28.75, 28.94, 29.55, 28.82, 29.51, 29.67, 29.45, 30.02, 30.06, 30.88, 30.84,
    31.27, 30.93, 31.15, 31.05, 30.53, 30.26, 30.64, 30.27, 30.19, 30.58, 30.1, 29.98, 30.4, 30.68,
    29.79, 28.83, 28.65, 29.13, 29.44, 29.01, 29.14, 28.14, 27.98, 27.89, 28.13, 28.12, 28.23,
    28.05, 28.03, 27.98, 28.07, 27.93, 28.24, 27.49, 27.32, 26.64, 26.52, 26.94, 27.45, 27.13,
    26.81, 26.49, 25.69, 25.6, 25.96, 25.49, 25.5, 25.49, 26.07, 26.74, 26.49, 26.51, 25.39, 25.61,
    25.3, 25.29, 25.65, 25.29, 24.29, 25.09, 25.22, 25.49, 25.56, 25.57, 25.48, 25.35, 25.13,
    25.73, 26.38, 26.32, 26.12, 26.07, 26.41, 26.54, 26.14, 26.91, 27.26, 27.11, 28.85, 28.75,
    28.58, 28.79, 28.78, 29.48, 29.82, 29.48, 28.89, 28.14, 27.85, 27.68, 26.91, 26.83, 27.38,
    27.07, 27.76, 27.92, 28.61, 28.61, 29.04, 29.08, 29.28, 29.16, 29.34, 29.6, 29.48, 28.91,
    28.77, 28.99, 29.19, 28.87, 29.16, 28.52, 28.25,
  ];

  let series_b: Vec<f64> = vec![
    9.36, 9.37, 9.25, 9.07, 9.13, 8.85, 8.86, 8.81, 8.49, 8.53, 8.26, 8.23, 8.1, 8.01, 8.19, 8.28,
    8.34, 8.25, 8.3, 8.48, 8.3, 8.47, 8.48, 8.29, 8.41, 8.52, 8.47, 8.42, 8.44, 8.28, 8.37, 8.41,
    8.33, 8.45, 8.55, 8.53, 8.57, 8.55, 8.42, 8.47, 8.0, 8.2, 8.07, 8.06, 8.16, 8.15, 7.98, 8.04,
    8.24, 8.27, 8.21, 8.25, 8.32, 8.39, 8.45, 8.39, 8.4, 8.44, 8.66, 8.61, 8.64, 8.57, 8.55, 8.67,
    8.67, 8.65, 8.67, 8.68, 8.77, 8.62, 8.54, 8.54, 8.52, 8.52, 8.48, 8.63, 8.67, 8.58, 8.46, 8.49,
    8.58, 8.78, 8.95, 8.86, 8.92, 8.68, 8.61, 8.75, 8.88, 8.98, 9.22, 9.09, 8.89, 8.74, 8.74, 8.7,
    8.92, 9.05, 9.15, 9.21, 9.19, 9.29, 9.25, 9.27, 8.93, 8.8, 8.84, 8.86, 9.0, 8.95, 9.08, 8.93,
    8.89, 8.84, 8.78, 8.75, 8.83, 8.74, 8.67, 8.69, 8.67, 8.66, 8.68, 8.75, 8.67, 8.9, 8.65, 8.68,
    8.68, 8.68, 8.57, 8.81, 8.84, 8.6, 8.43, 8.42, 8.4, 8.42, 8.42, 8.4, 8.23, 8.08, 7.97, 7.95,
    8.12, 8.03, 7.87, 7.8, 7.78, 7.92, 8.15, 7.88, 7.88, 7.91, 7.98, 8.07, 8.22, 8.25, 8.38, 8.31,
    8.12, 8.08, 7.89, 7.9, 8.14, 8.24, 8.23, 7.98, 8.02, 8.0, 8.05, 8.0, 8.0, 8.1, 7.97, 7.78,
    7.84, 7.84, 7.73, 7.65, 7.53, 7.55, 7.41, 7.38, 7.38, 7.48, 7.39, 7.49, 7.87, 7.9, 7.78, 7.64,
    7.64, 7.61, 7.94, 7.98, 7.98, 8.14, 8.19, 8.16, 8.39, 8.33, 8.33, 8.59, 8.56, 8.68, 8.57, 8.84,
    8.86, 8.96, 8.85, 9.09, 9.01, 9.04, 9.06, 9.18, 9.02, 8.87, 8.82, 8.7, 8.76, 8.61, 8.59, 8.74,
    8.85, 8.88, 8.78, 8.56, 8.58, 8.46, 8.57, 8.26, 8.24, 8.57, 8.49, 8.45, 8.45, 8.45, 8.25, 8.53,
    8.75, 8.57, 8.75, 8.91, 8.86, 8.73, 8.52, 8.53, 8.51, 8.74, 8.76, 8.9, 9.05, 8.86, 8.82, 8.75,
    8.82, 8.93, 9.03, 8.96, 9.01, 8.9, 8.93, 9.09, 9.1, 9.23, 9.3, 9.03, 8.96, 8.79, 8.75, 8.78,
    8.76, 8.62, 8.58, 8.75, 8.74, 8.47, 8.54, 8.69, 8.82, 8.73, 8.34, 8.38, 8.42, 8.28, 8.33, 8.48,
    8.3, 8.51, 8.53, 8.5, 8.63, 8.64, 8.88, 8.92, 8.99, 8.91, 8.93, 8.91, 8.76, 8.71, 8.8, 8.71,
    8.69, 8.76, 8.67, 8.61, 8.7, 8.76, 8.49, 8.2, 8.17, 8.3, 8.37, 8.24, 8.27, 8.0, 7.95, 8.0,
    8.04, 8.04, 8.09, 8.04, 8.04, 7.99, 8.09, 8.04, 8.15, 7.97, 7.88, 7.72, 7.67, 7.75, 7.96, 7.87,
    7.78, 7.69, 7.42, 7.4, 7.54, 7.39, 7.37, 7.39, 7.55, 7.74, 7.67, 7.7, 7.37, 7.43, 7.35, 7.34,
    7.44, 7.34, 7.01, 7.24, 7.33, 7.37, 7.34, 7.38, 7.35, 7.3, 7.19, 7.36, 7.56, 7.53, 7.46, 7.44,
    7.56, 7.58, 7.48, 7.65, 7.78, 7.7, 8.18, 8.21, 8.13, 8.19, 8.21, 8.38, 8.52, 8.43, 8.25, 8.05,
    7.97, 7.94, 7.71, 7.66, 7.82, 7.73, 7.92, 7.95, 8.14, 8.16, 8.27, 8.28, 8.34, 8.35, 8.43, 8.46,
    8.47, 8.32, 8.26, 8.37, 8.41, 8.33, 8.37, 8.23, 8.19,
  ];

  let points_a: Vec<(i32, f64)> = series_a
    .iter()
    .take(200)
    .enumerate()
    .map(|(idx, price)| (-(idx as i32), *price))
    .collect();

  let points_b: Vec<(i32, f64)> = series_b
    .iter()
    .take(200)
    .enumerate()
    .map(|(idx, price)| (-(idx as i32), *price))
    .collect();

  AppData {
    chart_data: LineChartData::new()
      .with_title("The quick brown fox jumped over the lazy dog.")
      .with_line(Line::new(points_a, Color::rgb8(0x73, 0xD0, 0xFF)))
      .with_line(Line::new(points_b, Color::rgb8(0xF2, 0x87, 0x79))),
    left: Arc::new(vec!["A".to_string(), "B".to_string()]),
  }
}
