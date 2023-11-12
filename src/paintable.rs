use std::convert::Infallible;

use gtk::{gdk, glib, graphene::Rect, gsk, pango, prelude::*, subclass::prelude::*};
use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
};

use crate::snapshot::SnapshotBackend;

mod imp {
    use std::cell::{OnceCell, RefCell};

    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::Paintable)]
    pub struct Paintable {
        #[property(get, set, construct_only)]
        pub(super) width: OnceCell<u32>,
        #[property(get, set, construct_only)]
        pub(super) height: OnceCell<u32>,

        pub(super) node: RefCell<Option<gsk::RenderNode>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Paintable {
        const NAME: &'static str = "PlottersGtk4Paintable";
        type Type = super::Paintable;
        type Interfaces = (gdk::Paintable,);
    }

    #[glib::derived_properties]
    impl ObjectImpl for Paintable {}

    impl PaintableImpl for Paintable {
        fn snapshot(&self, snapshot: &gdk::Snapshot, width: f64, height: f64) {
            let node = self.node.borrow();

            let Some(node) = node.as_ref() else {
                return;
            };

            snapshot.save();

            let (this_width, this_height) = self.obj().size();
            snapshot.scale(
                width as f32 / this_width as f32,
                height as f32 / this_height as f32,
            );

            snapshot.push_clip(&Rect::new(0.0, 0.0, this_width as f32, this_height as f32));

            snapshot.append_node(node);

            snapshot.pop();

            snapshot.restore();
        }

        fn flags(&self) -> gdk::PaintableFlags {
            gdk::PaintableFlags::SIZE
        }

        fn intrinsic_width(&self) -> i32 {
            *self.width.get().unwrap() as i32
        }

        fn intrinsic_height(&self) -> i32 {
            *self.height.get().unwrap() as i32
        }
    }
}

glib::wrapper! {
    /// A paintable to draw on in [`PaintableBackend`].
    pub struct Paintable(ObjectSubclass<imp::Paintable>)
        @implements gdk::Paintable;
}

impl Paintable {
    /// Creates a new paintable with the given width and height.
    pub fn new((w, h): (u32, u32)) -> Self {
        glib::Object::builder()
            .property("width", w)
            .property("height", h)
            .build()
    }

    /// Returns the size of the paintable.
    pub fn size(&self) -> (u32, u32) {
        let imp = self.imp();
        (*imp.width.get().unwrap(), *imp.height.get().unwrap())
    }

    /// Clears the contents of the paintable.
    pub fn clear(&self) {
        self.set_node(None);
    }

    fn set_node(&self, node: Option<gsk::RenderNode>) {
        self.imp().node.replace(node);
        self.invalidate_contents();
    }
}

/// A drawing backend backed with an object that implements `gdk::Paintable`.
#[derive(Debug)]
pub struct PaintableBackend<'a> {
    snapshot: Option<gtk::Snapshot>,
    paintable: &'a Paintable,
    layout: pango::Layout,
    size: (u32, u32),
}

impl<'a> PaintableBackend<'a> {
    /// Creates a new drawing backend backed with an object that implements
    /// `gdk::Paintable` with the given width and height.
    pub fn new(paintable: &'a Paintable) -> Self {
        let font_map = pangocairo::FontMap::default();
        let context = font_map.create_context();
        let layout = pango::Layout::new(&context);
        Self {
            snapshot: None,
            paintable,
            layout,
            size: paintable.size(),
        }
    }

    #[inline]
    fn inner(&self) -> SnapshotBackend<'_> {
        SnapshotBackend::from_parts(
            self.snapshot.as_ref().expect("backend was not prepared"),
            self.layout.clone(),
            self.size,
        )
    }
}

impl Drop for PaintableBackend<'_> {
    fn drop(&mut self) {
        if let Some(snapshot) = self.snapshot.take() {
            self.paintable.set_node(snapshot.to_node());
        }
    }
}

impl DrawingBackend for PaintableBackend<'_> {
    type ErrorType = Infallible;

    #[inline]
    fn get_size(&self) -> (u32, u32) {
        self.size
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if self.snapshot.is_none() {
            self.snapshot.replace(gtk::Snapshot::new());
        }
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if let Some(snapshot) = self.snapshot.take() {
            self.paintable.set_node(snapshot.to_node());
        }
        Ok(())
    }

    #[inline]
    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner().draw_pixel(point, color)
    }

    #[inline]
    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner().draw_line(from, to, style)
    }

    #[inline]
    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner()
            .draw_rect(upper_left, bottom_right, style, fill)
    }

    #[inline]
    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        raw_path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner().draw_path(raw_path, style)
    }

    #[inline]
    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner().fill_polygon(vert, style)
    }

    #[inline]
    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner().draw_circle(center, radius, style, fill)
    }

    #[inline]
    fn estimate_text_size<TStyle: BackendTextStyle>(
        &self,
        text: &str,
        style: &TStyle,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        self.inner().estimate_text_size(text, style)
    }

    #[inline]
    fn draw_text<TStyle: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner().draw_text(text, style, pos)
    }

    #[inline]
    fn blit_bitmap(
        &mut self,
        pos: BackendCoord,
        (iw, ih): (u32, u32),
        src: &[u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner().blit_bitmap(pos, (iw, ih), src)
    }
}
