use std::convert::Infallible;

use gtk::{pango, prelude::*};
use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
};

use crate::common;

/// Backend that draws to a [`gtk::Snapshot`].
#[derive(Debug)]
pub struct SnapshotBackend<'a> {
    snapshot: &'a gtk::Snapshot,
    layout: pango::Layout,
    size: (u32, u32),
}

impl<'a> SnapshotBackend<'a> {
    /// Creates a new drawing backend backed with [`gtk::Snapshot`] with
    /// the given width and height.
    pub fn new(snapshot: &'a gtk::Snapshot, (w, h): (u32, u32)) -> Self {
        let font_map = pangocairo::FontMap::default();
        let context = font_map.create_context();
        let layout = pango::Layout::new(&context);
        Self {
            snapshot,
            layout,
            size: (w, h),
        }
    }
}

impl<'a> DrawingBackend for SnapshotBackend<'a> {
    type ErrorType = Infallible;

    #[inline]
    fn get_size(&self) -> (u32, u32) {
        self.size
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    #[inline]
    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        common::draw_pixel(self.snapshot, point, color)
    }

    #[inline]
    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        common::draw_line(self.snapshot, from, to, style)
    }

    #[inline]
    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        common::draw_rect(self.snapshot, upper_left, bottom_right, style, fill)
    }

    #[inline]
    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        raw_path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        common::draw_path(self.snapshot, raw_path, style)
    }

    #[inline]
    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        common::fill_polygon(self.snapshot, vert, style)
    }

    #[inline]
    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        common::draw_circle(self.snapshot, center, radius, style, fill)
    }

    #[inline]
    fn estimate_text_size<TStyle: BackendTextStyle>(
        &self,
        text: &str,
        style: &TStyle,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        common::estimate_text_size(&self.layout, text, style)
    }

    #[inline]
    fn draw_text<TStyle: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        common::draw_text(self.snapshot, &self.layout, text, style, pos)
    }
}
