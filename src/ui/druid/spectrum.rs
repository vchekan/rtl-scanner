use std::sync::{Arc, Mutex};
use druid::{Widget, LifeCycle, EventCtx, PaintCtx, LifeCycleCtx, BoxConstraints, Size, LayoutCtx, Event, Env, UpdateCtx, RenderContext, Color};
use druid::kurbo::Line;
use log::debug;

pub struct Spectrum {

}

impl Default for Spectrum {
    fn default() -> Self { Spectrum { } }
}

impl Widget<Arc<Mutex<Vec<f64>>>> for Spectrum {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Arc<Mutex<Vec<f64>>>, env: &Env) { /* Empty */ }
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Arc<Mutex<Vec<f64>>>, env: &Env) { /* Empty */ }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Arc<Mutex<Vec<f64>>>, data: &Arc<Mutex<Vec<f64>>>, env: &Env) { /* Empty */ }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &Arc<Mutex<Vec<f64>>>, env: &Env) -> Size {
        bc.max() // take all space available
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Arc<Mutex<Vec<f64>>>, env: &Env) {
        let size = ctx.size();
        let rect = size.to_rect();
        ctx.fill(rect, &Color::rgb(0.8, 0.8, 0.8));

        let data = data.lock().unwrap();
        let scaled = crate::charts::rescale(size.width as i32, size.height as i32, &data);
        let brush = Color::BLUE;
        for i in 0..scaled.len() {
            let f = scaled[i];
            let line = Line::new((i as f64, 0_f64), (i as f64, f as f64));
            ctx.stroke(line, &brush, 1.0);
        }
        debug!("painted {}", data.len());
    }
}