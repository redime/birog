use druid::widget::{Flex, Label, ListIter};
use druid::{
  BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point, Rect, Size,
  UpdateCtx, Widget, WidgetPod,
};
use std::cmp::Ordering;

pub struct Table<T>
where
  T: Data,
{
  label_fn: fn(&T) -> String,
  cursor_pos: Point,
  children: Vec<WidgetPod<T, Label<T>>>,
}

impl<T> Table<T>
where
  T: Data,
{
  pub fn new(label_fn: fn(&T) -> String) -> Self {
    Self {
      label_fn,
      cursor_pos: Point::ORIGIN,
      children: Vec::new(),
    }
  }

  fn update_child_count(&mut self, data: &impl ListIter<T>, _env: &Env) -> bool {
    let len = self.children.len();
    match len.cmp(&data.data_len()) {
      Ordering::Greater => self.children.truncate(data.data_len()),
      Ordering::Less => data.for_each(|t, i| {
        if i >= len {
          let lbl = (self.label_fn)(t);
          let child = WidgetPod::new(Label::new(lbl));
          self.children.push(child);
        }
      }),
      Ordering::Equal => (),
    }

    len != data.data_len()
  }
}

impl<T> Widget<T> for Table<String>
where
  T: ListIter<String>,
{
  fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
    match event {
      Event::MouseDown(e) => data.for_each_mut(|t, _| {
        t.push('a');
        ctx.children_changed();
        ctx.request_paint();
      }),
      _ => (),
    }

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

  fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
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
    let mut children = self.children.iter_mut();
    data.for_each(|child_data, _| {
      if let Some(child) = children.next() {
        child.paint(ctx, child_data, env);
      }
    });
  }
}
