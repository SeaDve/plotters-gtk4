use std::convert::Infallible;

use gtk::{
    gdk,
    graphene::{Point, Rect},
    gsk, pango,
    prelude::*,
};
use plotters_backend::{
    text_anchor::{HPos, VPos},
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingErrorKind, FontStyle,
    FontTransform,
};

const FILL_RULE: gsk::FillRule = gsk::FillRule::Winding;

pub fn draw_pixel(
    snapshot: &gtk::Snapshot,
    point: BackendCoord,
    color: BackendColor,
) -> Result<(), DrawingErrorKind<Infallible>> {
    snapshot.append_color(
        &color.to_rgba(),
        &Rect::new(point.0 as f32, point.1 as f32, 1.0, 1.0),
    );
    Ok(())
}

pub fn draw_line<S: BackendStyle>(
    snapshot: &gtk::Snapshot,
    from: BackendCoord,
    to: BackendCoord,
    style: &S,
) -> Result<(), DrawingErrorKind<Infallible>> {
    let path_builder = gsk::PathBuilder::new();
    path_builder.move_to(from.0 as f32, from.1 as f32);
    path_builder.line_to(to.0 as f32, to.1 as f32);
    let path = path_builder.to_path();

    let stroke = gsk::Stroke::new(style.stroke_width() as f32);
    snapshot.append_stroke(&path, &stroke, &style.color().to_rgba());

    Ok(())
}

pub fn draw_rect<S: BackendStyle>(
    snapshot: &gtk::Snapshot,
    upper_left: BackendCoord,
    bottom_right: BackendCoord,
    style: &S,
    fill: bool,
) -> Result<(), DrawingErrorKind<Infallible>> {
    let bounds = Rect::new(
        upper_left.0 as f32,
        upper_left.1 as f32,
        (bottom_right.0 - upper_left.0) as f32,
        (bottom_right.1 - upper_left.1) as f32,
    );
    if fill {
        snapshot.append_color(&style.color().to_rgba(), &bounds);
    } else {
        snapshot.append_border(
            &gsk::RoundedRect::from_rect(bounds, 0.0),
            &[style.stroke_width() as f32; 4],
            &[style.color().to_rgba(); 4],
        );
    }

    Ok(())
}

pub fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
    snapshot: &gtk::Snapshot,
    raw_path: I,
    style: &S,
) -> Result<(), DrawingErrorKind<Infallible>> {
    let mut raw_path_iter = raw_path.into_iter();
    if let Some((x, y)) = raw_path_iter.next() {
        let path_builder = gsk::PathBuilder::new();

        path_builder.move_to(x as f32, y as f32);

        for (x, y) in raw_path_iter {
            path_builder.line_to(x as f32, y as f32);
        }

        let path = path_builder.to_path();

        let stroke = gsk::Stroke::new(style.stroke_width() as f32);
        snapshot.append_stroke(&path, &stroke, &style.color().to_rgba());
    }

    Ok(())
}

pub fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
    snapshot: &gtk::Snapshot,
    vert: I,
    style: &S,
) -> Result<(), DrawingErrorKind<Infallible>> {
    let mut vert_iter = vert.into_iter();
    if let Some((x, y)) = vert_iter.next() {
        let path_builder = gsk::PathBuilder::new();

        path_builder.move_to(x as f32, y as f32);

        for (x, y) in vert_iter {
            path_builder.line_to(x as f32, y as f32);
        }

        path_builder.close();
        let path = path_builder.to_path();

        snapshot.append_fill(&path, FILL_RULE, &style.color().to_rgba());
    }

    Ok(())
}

pub fn draw_circle<S: BackendStyle>(
    snapshot: &gtk::Snapshot,
    center: BackendCoord,
    radius: u32,
    style: &S,
    fill: bool,
) -> Result<(), DrawingErrorKind<Infallible>> {
    let path_builder = gsk::PathBuilder::new();
    path_builder.add_circle(&Point::new(center.0 as f32, center.1 as f32), radius as f32);
    let path = path_builder.to_path();

    if fill {
        snapshot.append_fill(&path, FILL_RULE, &style.color().to_rgba());
    } else {
        let stroke = gsk::Stroke::new(style.stroke_width() as f32);
        snapshot.append_stroke(&path, &stroke, &style.color().to_rgba());
    }

    Ok(())
}

pub fn estimate_text_size<TStyle: BackendTextStyle>(
    layout: &pango::Layout,
    text: &str,
    style: &TStyle,
) -> Result<(u32, u32), DrawingErrorKind<Infallible>> {
    layout.set_text(text);
    layout_set_style(layout, style);

    let (width, height) = layout.pixel_size();
    Ok((width as u32, height as u32))
}

pub fn draw_text<TStyle: BackendTextStyle>(
    snapshot: &gtk::Snapshot,
    layout: &pango::Layout,
    text: &str,
    style: &TStyle,
    pos: BackendCoord,
) -> Result<(), DrawingErrorKind<Infallible>> {
    layout.set_text(text);
    layout_set_style(layout, style);

    snapshot.save();

    let (_, extents) = layout.pixel_extents();
    let dx = match style.anchor().h_pos {
        HPos::Left => 0.0,
        HPos::Center => -extents.width() as f32 / 2.0,
        HPos::Right => -extents.width() as f32,
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
        snapshot.translate(&Point::new(
            pos.0 as f32 + dx,
            pos.1 as f32 + dy - extents.height() as f32,
        ));
    } else {
        snapshot.translate(&Point::new(pos.0 as f32, pos.1 as f32));
        snapshot.rotate(rotate);
        snapshot.translate(&Point::new(dx, dy - extents.height() as f32));
    }

    snapshot.append_layout(layout, &style.color().to_rgba());

    snapshot.restore();

    Ok(())
}

fn layout_set_style(layout: &pango::Layout, style: &impl BackendTextStyle) {
    let mut font_desc = pango::FontDescription::new();
    font_desc.set_family(style.family().as_str());
    font_desc.set_absolute_size(style.size() * pango::SCALE as f64);
    match style.style() {
        FontStyle::Normal => font_desc.set_style(pango::Style::Normal),
        FontStyle::Bold => font_desc.set_weight(pango::Weight::Bold),
        FontStyle::Italic => font_desc.set_style(pango::Style::Italic),
        FontStyle::Oblique => font_desc.set_style(pango::Style::Oblique),
    }
    layout.set_font_description(Some(&font_desc));
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
