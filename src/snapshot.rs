use std::convert::Infallible;

use gtk::{
    gdk,
    graphene::{Point, Rect},
    gsk, pango,
    prelude::*,
};
use plotters_backend::text_anchor::{HPos, VPos};
use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
    FontStyle, FontTransform,
};

const FILL_RULE: gsk::FillRule = gsk::FillRule::EvenOdd;

/// Backend that draws to a `GtkSnapshot`.
#[derive(Debug)]
pub struct SnapshotBackend<'a> {
    snapshot: &'a gtk::Snapshot,
    layout: pango::Layout,
    width: u32,
    height: u32,
}

impl<'a> SnapshotBackend<'a> {
    /// Creates a new drawing backend backed with `GtkSnapshot` with
    /// the given width and height.
    pub fn new(snapshot: &'a gtk::Snapshot, (width, height): (u32, u32)) -> Self {
        let context = pangocairo::FontMap::default().create_context();
        let layout = pango::Layout::new(&context);

        Self {
            snapshot,
            layout,
            width,
            height,
        }
    }

    fn set_layout_style(&self, style: &impl BackendTextStyle) {
        let mut font_desc = pango::FontDescription::new();
        font_desc.set_family(style.family().as_str());
        font_desc.set_absolute_size(style.size() * pango::SCALE as f64);
        match style.style() {
            FontStyle::Normal => font_desc.set_style(pango::Style::Normal),
            FontStyle::Bold => font_desc.set_weight(pango::Weight::Bold),
            FontStyle::Italic => font_desc.set_style(pango::Style::Italic),
            FontStyle::Oblique => font_desc.set_style(pango::Style::Oblique),
        }
        self.layout.set_font_description(Some(&font_desc));
    }
}

impl<'a> DrawingBackend for SnapshotBackend<'a> {
    type ErrorType = Infallible;

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.snapshot.append_color(
            &color.to_rgba(),
            &Rect::new(point.0 as f32, point.1 as f32, 1.0, 1.0),
        );
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let path_builder = gsk::PathBuilder::new();
        path_builder.move_to(from.0 as f32, from.1 as f32);
        path_builder.line_to(to.0 as f32, to.1 as f32);
        let path = path_builder.to_path();

        let stroke = gsk::Stroke::new(style.stroke_width() as f32);
        self.snapshot
            .append_stroke(&path, &stroke, &style.color().to_rgba());

        Ok(())
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let bounds = Rect::new(
            upper_left.0 as f32,
            upper_left.1 as f32,
            (bottom_right.0 - upper_left.0) as f32,
            (bottom_right.1 - upper_left.1) as f32,
        );
        if fill {
            self.snapshot
                .append_color(&style.color().to_rgba(), &bounds);
        } else {
            self.snapshot.append_border(
                &gsk::RoundedRect::from_rect(bounds, 0.0),
                &[style.stroke_width() as f32; 4],
                &[style.color().to_rgba(); 4],
            );
        }

        Ok(())
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        raw_path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let mut raw_path_iter = raw_path.into_iter();
        if let Some((x, y)) = raw_path_iter.next() {
            let path_builder = gsk::PathBuilder::new();

            path_builder.move_to(x as f32, y as f32);

            for (x, y) in raw_path_iter {
                path_builder.line_to(x as f32, y as f32);
            }

            let path = path_builder.to_path();

            let stroke = gsk::Stroke::new(style.stroke_width() as f32);
            self.snapshot
                .append_stroke(&path, &stroke, &style.color().to_rgba());
        }

        Ok(())
    }

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let mut vert_iter = vert.into_iter();
        if let Some((x, y)) = vert_iter.next() {
            let path_builder = gsk::PathBuilder::new();

            path_builder.move_to(x as f32, y as f32);

            for (x, y) in vert_iter {
                path_builder.line_to(x as f32, y as f32);
            }

            let path = path_builder.to_path();

            self.snapshot
                .append_fill(&path, FILL_RULE, &style.color().to_rgba());
        }

        Ok(())
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let path_builder = gsk::PathBuilder::new();
        path_builder.add_circle(&Point::new(center.0 as f32, center.1 as f32), radius as f32);
        let path = path_builder.to_path();

        if fill {
            self.snapshot
                .append_fill(&path, FILL_RULE, &style.color().to_rgba());
        } else {
            let stroke = gsk::Stroke::new(style.stroke_width() as f32);
            self.snapshot
                .append_stroke(&path, &stroke, &style.color().to_rgba());
        }

        Ok(())
    }

    fn estimate_text_size<TStyle: BackendTextStyle>(
        &self,
        text: &str,
        style: &TStyle,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        self.layout.set_text(text);
        self.set_layout_style(style);

        let (width, height) = self.layout.pixel_size();
        Ok((width as u32, height as u32))
    }

    fn draw_text<TStyle: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.layout.set_text(text);
        self.set_layout_style(style);

        self.snapshot.save();

        let (_, extents) = self.layout.pixel_extents();
        let dx = match style.anchor().h_pos {
            HPos::Left => 0.0,
            HPos::Right => -extents.width() as f32,
            HPos::Center => -extents.width() as f32 / 2.0,
        };
        let dy = match style.anchor().v_pos {
            VPos::Top => extents.height() as f32,
            VPos::Center => extents.height() as f32 / 2.0,
            VPos::Bottom => 0.0,
        };

        let rotate = match style.transform() {
            FontTransform::None => 0.0,
            FontTransform::Rotate90 => 90.0,
            FontTransform::Rotate180 => 180.0,
            FontTransform::Rotate270 => 270.0,
        };
        if rotate == 0.0 {
            self.snapshot.translate(&Point::new(
                pos.0 as f32 + dx,
                pos.1 as f32 + dy - extents.height() as f32,
            ));
        } else {
            self.snapshot
                .translate(&Point::new(pos.0 as f32, pos.1 as f32));
            self.snapshot.rotate(rotate);
            self.snapshot
                .translate(&Point::new(dx, dy - extents.height() as f32));
        }

        self.snapshot
            .append_layout(&self.layout, &style.color().to_rgba());

        self.snapshot.restore();

        Ok(())
    }
}

trait BackendColorExt {
    fn to_rgba(&self) -> gdk::RGBA;
}

impl BackendColorExt for BackendColor {
    fn to_rgba(&self) -> gdk::RGBA {
        gdk::RGBA::new(
            self.rgb.0 as f32 / 255.0,
            self.rgb.1 as f32 / 255.0,
            self.rgb.2 as f32 / 255.0,
            self.alpha as f32,
        )
    }
}
