use birog::table::Table;
use druid::widget::{CrossAxisAlignment, Flex, Label};
use druid::{AppLauncher, Data, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};
use std::sync::Arc;

#[derive(Clone, Data, Lens)]
struct AppData {
  data: Arc<Vec<Person>>,
}

#[derive(Clone, Data, Lens)]
struct Person {
  first_name: String,
  last_name: String,
  phone: String,
}

fn main() {
  let window = WindowDesc::new(ui_builder)
    .window_size((1024., 500.))
    .title(LocalizedString::new("custom-widget-demo-window-title").with_placeholder("Table Test"));

  AppLauncher::with_window(window)
    .use_simple_logger()
    .launch(data_builder())
    .expect("launch failed");
}

fn ui_builder() -> impl Widget<AppData> {
  Table::new()
    .with_column(
      "First Name",
      || {
        Flex::column()
          .with_child(Label::new(|person: &Person, _env: &_| {
            person.first_name.clone()
          }))
          .with_child(Label::new(|person: &Person, _env: &_| {
            person.first_name.clone()
          }))
          .cross_axis_alignment(CrossAxisAlignment::Start)
      },
      200.0,
    )
    .with_column(
      "Last Name",
      || Label::new(|person: &Person, _env: &_| person.last_name.clone()),
      200.0,
    )
    .with_column(
      "Phone Number",
      || Label::new(|person: &Person, _env: &_| person.phone.clone()),
      100.0,
    )
    .lens(AppData::data)
    .debug_paint_layout()
}

fn data_builder() -> AppData {
  AppData {
    data: Arc::new(vec![
      Person {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        phone: "555-1234".to_string(),
      },
      Person {
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        phone: "555-0000".to_string(),
      },
      Person {
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        phone: "555-0000".to_string(),
      },
    ]),
  }
}
