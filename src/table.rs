use std::marker::PhantomData;

use druid::widget::{Flex, Label, LabelText, List, ListIter, SizedBox};
use druid::{
  BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point,
  Rect, Size, UpdateCtx, Widget, WidgetPod,
};

pub struct Table<C, T> {
  root: WidgetPod<T, Flex<T>>,
  data_type: PhantomData<C>,
}

impl<C: Data, T: ListIter<C>> Table<C, T> {
  pub fn new() -> Self {
    Self {
      root: WidgetPod::new(Flex::row()),
      data_type: Default::default(),
    }
  }

  pub fn with_column<W: Widget<C> + 'static>(
    mut self,
    title: impl Into<LabelText<T>>,
    closure: impl Fn() -> W + 'static,
    width: f64,
  ) -> Self {
    let widget = Flex::column()
      .with_child(SizedBox::new(Label::new(title)).width(width))
      .with_child(SizedBox::new(List::new(closure)).width(width));

    self.root.widget_mut().add_child(widget);

    self
  }

  pub fn with_flex_column<W: Widget<C> + 'static>(
    mut self,
    title: impl Into<LabelText<T>>,
    closure: impl Fn() -> W + 'static,
    param: f64,
  ) -> Self {
    let widget = Flex::column()
      .with_child(Label::new(title))
      .with_child(List::new(closure));

    self.root.widget_mut().add_flex_child(widget, param);

    self
  }
}

impl<C: Data, T: ListIter<C>> Widget<T> for Table<C, T> {
  fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
    self.root.event(ctx, event, data, env)
  }

  fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
    self.root.lifecycle(ctx, event, data, env)
  }

  fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
    self.root.update(ctx, data, env)
  }

  fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
    let mut width = bc.min().width;
    let mut height = 0.0;

    let mut paint_rect = Rect::ZERO;

    let child_bc = BoxConstraints::new(
      Size::new(bc.min().width, 0.0),
      Size::new(bc.max().width, std::f64::INFINITY),
    );
    let child_size = self.root.layout(ctx, &child_bc, data, env);
    let rect = Rect::from_origin_size(Point::new(0.0, height), child_size);
    self.root.set_layout_rect(ctx, data, env, rect);
    paint_rect = paint_rect.union(self.root.paint_rect());

    width = width.max(child_size.width);
    height = child_size.height;

    let my_size = bc.constrain(Size::new(width, height));
    let insets = paint_rect - Rect::ZERO.with_size(my_size);
    ctx.set_paint_insets(insets);
    my_size
  }

  fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
    self.root.paint(ctx, data, env)
  }
}
