use super::Message;
use iced_native::{widget::Widget, renderer::Renderer, Hasher, Layout, Length, Point};
use iced_native::layout::{Limits, Node};

pub struct SpectrumChart {

}

impl SpectrumChart {
    pub fn new() -> Self {
        SpectrumChart {
            
        }
    }
}

impl<M, R: Renderer> Widget<M, R> for SpectrumChart {
    fn width(&self) -> Length {
        unimplemented!()
    }

    fn height(&self) -> Length {
        unimplemented!()
    }

    fn layout(&self, renderer: &R, limits: &Limits) -> Node {
        unimplemented!()
    }

    fn draw(&self, renderer: &mut R, layout: Layout<'_>, cursor_position: Point) -> R::Output {
        unimplemented!()
    }

    fn hash_layout(&self, state: &mut Hasher) {
        unimplemented!()
    }
}