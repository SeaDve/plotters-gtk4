use std::convert::Infallible;

use gtk::{gdk, glib, graphene::Rect, gsk, pango, prelude::*, subclass::prelude::*};
use pangocairo::prelude::*;
use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
};

use crate::common;

mod imp {
    use std::{
        cell::{OnceCell, RefCell},
        sync::OnceLock,
    };

    use super::*;

    #[derive(Debug, Default)]
    pub struct Paintable {
        pub(super) width: OnceCell<u32>,
        pub(super) height: OnceCell<u32>,
        pub(super) node: RefCell<Option<gsk::RenderNode>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Paintable {
        const NAME: &'static str = "PlottersGtk4Paintable";
        type Type = super::Paintable;
        type Interfaces = (gdk::Paintable,);
    }

    impl ObjectImpl for Paintable {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();

            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecUInt::builder("width")
                        .maximum(i32::MAX as u32)
                        .construct_only()
                        .build(),
                    glib::ParamSpecUInt::builder("height")
                        .maximum(i32::MAX as u32)
                        .construct_only()
                        .build(),
                ]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "width" => {
                    let width = value.get().unwrap();
                    self.width.set(width).unwrap();
                }
                "height" => {
                    let height = value.get().unwrap();
                    self.height.set(height).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "width" => self.obj().width().into(),
                "height" => self.obj().height().into(),
                _ => unimplemented!(),
            }
        }
    }

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
            self.obj().width() as i32
        }

        fn intrinsic_height(&self) -> i32 {
            self.obj().height() as i32
        }
    }
}

glib::wrapper! {
    /// A paintable to draw on in [`PaintableBackend`].
    ///
    /// This can be used on GTK UI files using its type name `PlottersGtk4Paintable`.
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

    /// Returns the width of the paintable.
    pub fn width(&self) -> u32 {
        *self.imp().width.get().unwrap()
    }

    /// Returns the height of the paintable.
    pub fn height(&self) -> u32 {
        *self.imp().height.get().unwrap()
    }

    /// Returns the width and height of the paintable.
    pub fn size(&self) -> (u32, u32) {
        (self.width(), self.height())
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

/// Backend that draws to an object that implements [`gdk::Paintable`].
#[derive(Debug)]
pub struct PaintableBackend<'a> {
    snapshot: Option<gtk::Snapshot>,
    paintable: &'a Paintable,
    layout: pango::Layout,
    size: (u32, u32),
}

impl<'a> PaintableBackend<'a> {
    /// Creates a new drawing backend backed with an object that implements
    /// [`gdk::Paintable`].
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
    fn snapshot(&self) -> &gtk::Snapshot {
        self.snapshot.as_ref().expect("backend was not prepared")
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
        common::draw_pixel(self.snapshot(), point, color)
    }

    #[inline]
    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        common::draw_line(self.snapshot(), from, to, style)
    }

    #[inline]
    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        common::draw_rect(self.snapshot(), upper_left, bottom_right, style, fill)
    }

    #[inline]
    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        raw_path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        common::draw_path(self.snapshot(), raw_path, style)
    }

    #[inline]
    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        common::fill_polygon(self.snapshot(), vert, style)
    }

    #[inline]
    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        common::draw_circle(self.snapshot(), center, radius, style, fill)
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
        common::draw_text(self.snapshot(), &self.layout, text, style, pos)
    }
}
