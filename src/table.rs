use std::cmp::Ordering;

use druid::widget::{Flex, Label, LabelText, ListIter, SizedBox};
use druid::{
  BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point,
  Rect, Size, UpdateCtx, Widget, WidgetPod,
};

pub struct Table<T> {
  headers: WidgetPod<(), Flex<()>>,
  columns: Vec<Column<T>>,
  children: Vec<WidgetPod<T, Flex<T>>>,
}

struct Column<T> {
  widget: Box<dyn Fn() -> Box<dyn Widget<T>>>,
  width: f64,
}

impl<T: Data> Table<T> {
  pub fn new() -> Self {
    Self {
      headers: WidgetPod::new(Flex::row()),
      columns: Vec::new(),
      children: Vec::new(),
    }
  }

  pub fn with_column<H: Widget<()> + 'static, W: Widget<T> + 'static>(
    mut self,
    header: H,
    closure: impl Fn() -> W + 'static,
    width: f64,
  ) -> Self {
    self
      .headers
      .widget_mut()
      .add_child(SizedBox::new(header).width(width));

    self.columns.push(Column {
      widget: Box::new(move || Box::new((closure)())),
      width,
    });

    self
  }

  fn update_child_count(&mut self, data: &impl ListIter<T>, _env: &Env) -> bool {
    let len = self.children.len();
    match len.cmp(&data.data_len()) {
      Ordering::Greater => self.children.truncate(data.data_len()),
      Ordering::Less => data.for_each(|_, i| {
        if i >= len {
          let mut widget = Flex::row();

          for column in self.columns.iter() {
            let child = (column.widget)();
            widget.add_child(SizedBox::new(child).width(column.width));
          }

          self.children.push(WidgetPod::new(widget));
        }
      }),
      Ordering::Equal => (),
    }
    len != data.data_len()
  }
}

impl<C: Data, T: ListIter<C>> Widget<T> for Table<C> {
  fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
    let mut children = self.children.iter_mut();
    data.for_each_mut(|child_data, _| {
      if let Some(child) = children.next() {
        child.event(ctx, event, child_data, env);
      }
    });
  }

  fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
    if let LifeCycle::WidgetAdded = event {
      if self.update_child_count(data, env) {
        ctx.children_changed();
      }
    }

    let mut children = self.children.iter_mut();
    data.for_each(|child_data, _| {
      if let Some(child) = children.next() {
        child.lifecycle(ctx, event, child_data, env);
      }
    });
  }

  fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
    // we send update to children first, before adding or removing children;
    // this way we avoid sending update to newly added children, at the cost
    // of potentially updating children that are going to be removed.
    let mut children = self.children.iter_mut();
    data.for_each(|child_data, _| {
      if let Some(child) = children.next() {
        child.update(ctx, child_data, env);
      }
    });

    if self.update_child_count(data, env) {
      ctx.children_changed();
    }
  }

  fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
    let mut width = bc.min().width;
    let mut y = 0.0;
    let mut paint_rect = Rect::ZERO;

    let header_bc = BoxConstraints::new(
      Size::new(bc.min().width, 0.0),
      Size::new(bc.max().width, std::f64::INFINITY),
    );

    let header_size = self.headers.layout(ctx, &header_bc, &(), env);
    let rect = Rect::from_origin_size(Point::new(0.0, y), header_size);
    self.headers.set_layout_rect(ctx, &(), env, rect);
    paint_rect = paint_rect.union(self.headers.paint_rect());
    width = width.max(header_size.width);
    y += header_size.height;

    let mut children = self.children.iter_mut();
    data.for_each(|child_data, _| {
      let child = match children.next() {
        Some(child) => child,
        None => {
          return;
        }
      };
      let child_bc = BoxConstraints::new(
        Size::new(bc.min().width, 0.0),
        Size::new(bc.max().width, std::f64::INFINITY),
      );
      let child_size = child.layout(ctx, &child_bc, child_data, env);
      let rect = Rect::from_origin_size(Point::new(0.0, y), child_size);
      child.set_layout_rect(ctx, child_data, env, rect);
      paint_rect = paint_rect.union(child.paint_rect());
      width = width.max(child_size.width);
      y += child_size.height;
    });

    let my_size = bc.constrain(Size::new(width, y));
    let insets = paint_rect - Rect::ZERO.with_size(my_size);
    ctx.set_paint_insets(insets);
    my_size
  }

  fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
    self.headers.paint(ctx, &(), env);

    let mut children = self.children.iter_mut();
    data.for_each(|child_data, _| {
      if let Some(child) = children.next() {
        child.paint(ctx, child_data, env);
      }
    });
  }
}
