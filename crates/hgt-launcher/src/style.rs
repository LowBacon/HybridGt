//! HybridGt's look: a dark night with a neon sunset, rounded corners, soft hover glows.

use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, Response, RichText, Rounding, Sense, Shape, Stroke, Ui, Vec2,
};

pub const BACKGROUND: Color32 = Color32::from_rgb(13, 10, 22);
pub const PANEL: Color32 = Color32::from_rgb(19, 14, 31);
pub const CARD: Color32 = Color32::from_rgb(26, 20, 42);
pub const CARD_HOVER: Color32 = Color32::from_rgb(36, 27, 58);
pub const EDGE: Color32 = Color32::from_rgb(50, 39, 76);
pub const ACCENT: Color32 = Color32::from_rgb(255, 79, 139);
pub const SUN: Color32 = Color32::from_rgb(255, 170, 72);
pub const VIOLET: Color32 = Color32::from_rgb(124, 58, 237);
pub const CYAN: Color32 = Color32::from_rgb(56, 214, 236);
pub const STOP: Color32 = Color32::from_rgb(232, 72, 72);
pub const SOFT: Color32 = Color32::from_rgb(196, 186, 220);
pub const MUTED: Color32 = Color32::from_rgb(132, 120, 160);

const ROUND: f32 = 14.0;

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()), l(a.a(), b.a()))
}

fn with_alpha(c: Color32, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (a.clamp(0.0, 1.0) * 255.0) as u8)
}

pub fn apply(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = BACKGROUND;
    v.window_fill = PANEL;
    v.extreme_bg_color = Color32::from_rgb(10, 8, 17);
    v.faint_bg_color = CARD;
    v.selection.bg_fill = with_alpha(ACCENT, 0.55);
    v.selection.stroke = Stroke::new(1.0_f32, ACCENT);
    v.window_rounding = Rounding::same(ROUND);
    for (w, fill, stroke) in [
        (&mut v.widgets.noninteractive, CARD, EDGE),
        (&mut v.widgets.inactive, CARD_HOVER, EDGE),
        (&mut v.widgets.hovered, Color32::from_rgb(48, 36, 76), ACCENT),
        (&mut v.widgets.active, ACCENT, ACCENT),
        (&mut v.widgets.open, CARD_HOVER, VIOLET),
    ] {
        w.rounding = Rounding::same(10.0);
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.bg_stroke = Stroke::new(1.0_f32, stroke);
    }
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, SOFT);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    v.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
    v.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.button_padding = Vec2::new(14.0, 8.0);
        s.spacing.item_spacing = Vec2::new(10.0, 8.0);
        s.spacing.slider_width = 220.0;
        s.text_styles.insert(egui::TextStyle::Body, FontId::proportional(15.0));
        s.text_styles.insert(egui::TextStyle::Button, FontId::proportional(15.0));
        s.text_styles.insert(egui::TextStyle::Small, FontId::proportional(12.0));
        s.text_styles.insert(egui::TextStyle::Heading, FontId::proportional(26.0));
        s.animation_time = 0.14;
    });
}

/// A horizontal gradient through `stops` across `rect`.
fn gradient(painter: &egui::Painter, rect: Rect, stops: &[Color32]) {
    let mut mesh = egui::Mesh::default();
    let n = stops.len().max(2) - 1;
    for (i, c) in stops.iter().enumerate() {
        let x = rect.left() + rect.width() * i as f32 / n as f32;
        mesh.colored_vertex(Pos2::new(x, rect.top()), *c);
        mesh.colored_vertex(Pos2::new(x, rect.bottom()), *c);
    }
    for i in 0..n as u32 {
        let k = i * 2;
        mesh.add_triangle(k, k + 1, k + 2);
        mesh.add_triangle(k + 1, k + 2, k + 3);
    }
    painter.add(Shape::mesh(mesh));
}

/// The brand mark: a sunset disc with dark bands (drawn here; no art of any game).
pub fn mark(painter: &egui::Painter, center: Pos2, r: f32) {
    let steps = 24;
    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        let y0 = center.y - r + 2.0 * r * i as f32 / steps as f32;
        let y1 = y0 + 2.0 * r / steps as f32 + 0.5;
        let c = if t < 0.5 { mix(SUN, ACCENT, t * 2.0) } else { mix(ACCENT, VIOLET, (t - 0.5) * 2.0) };
        // The disc's width at this height.
        let dy = ((y0 + y1) / 2.0 - center.y).abs().min(r);
        let half = (r * r - dy * dy).sqrt();
        let band = i > steps / 2 && (i - steps / 2) % 3 == 0;
        if !band {
            painter.rect_filled(Rect::from_min_max(Pos2::new(center.x - half, y0), Pos2::new(center.x + half, y1)), Rounding::ZERO, c);
        }
    }
}

/// The sidebar's top: the mark and the name.
pub fn brand(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 64.0), Sense::hover());
    let painter = ui.painter();
    mark(painter, Pos2::new(rect.left() + 26.0, rect.center().y), 18.0);
    painter.text(Pos2::new(rect.left() + 56.0, rect.center().y - 8.0), Align2::LEFT_CENTER, "HYBRIDGT", FontId::proportional(22.0), Color32::WHITE);
    painter.text(Pos2::new(rect.left() + 57.0, rect.center().y + 14.0), Align2::LEFT_CENTER, "built on VIctory", FontId::proportional(11.5), MUTED);
}

/// A hero banner: the title's own art if there is one (shown as it is, dimmed on the left
/// for the text), else a drawn sunset that drifts slowly. Returns the banner's rect.
pub fn hero(ui: &mut Ui, art: Option<&egui::TextureHandle>, t: f32, animate: bool, height: f32) -> Rect {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let painter = ui.painter_at(rect);
    let round = Rounding::same(22.0);
    match art {
        Some(tex) => {
            // Cover the banner, keeping the image's shape, centred.
            let size = tex.size_vec2();
            let scale = (rect.width() / size.x).max(rect.height() / size.y);
            let shown = size * scale;
            let uv_w = rect.width() / shown.x;
            let uv_h = rect.height() / shown.y;
            let uv = Rect::from_min_max(Pos2::new((1.0 - uv_w) / 2.0, (1.0 - uv_h) / 2.0), Pos2::new((1.0 + uv_w) / 2.0, (1.0 + uv_h) / 2.0));
            let mut shape = egui::epaint::RectShape::filled(rect, round, Color32::WHITE);
            shape.fill_texture_id = tex.id();
            shape.uv = uv;
            painter.add(shape);
        }
        None => {
            let drift = if animate { (t * 0.25).sin() * 0.5 + 0.5 } else { 0.5 };
            let stops = [mix(VIOLET, Color32::from_rgb(70, 24, 120), drift), mix(ACCENT, Color32::from_rgb(255, 92, 120), 1.0 - drift), SUN, Color32::from_rgb(40, 18, 70)];
            gradient(&painter, rect, &stops);
            mark(&painter, Pos2::new(rect.right() - 150.0, rect.center().y), 90.0);
        }
    }
    // A veil from the left and the bottom, so the text reads on any art.
    let mut veil = egui::Mesh::default();
    let mid = rect.left() + rect.width() * 0.62;
    veil.colored_vertex(rect.left_top(), with_alpha(BACKGROUND, 0.92));
    veil.colored_vertex(Pos2::new(mid, rect.top()), Color32::TRANSPARENT);
    veil.colored_vertex(rect.left_bottom(), with_alpha(BACKGROUND, 0.95));
    veil.colored_vertex(Pos2::new(mid, rect.bottom()), with_alpha(BACKGROUND, 0.25));
    veil.add_triangle(0, 1, 2);
    veil.add_triangle(1, 2, 3);
    painter.add(Shape::mesh(veil));
    painter.rect_stroke(rect, round, Stroke::new(1.0_f32, EDGE));
    rect
}

/// A small rounded label in a colour (a status).
pub fn chip(ui: &mut Ui, text: &str, color: Color32) {
    egui::Frame::none()
        .fill(with_alpha(color, 0.18))
        .stroke(Stroke::new(1.0_f32, with_alpha(color, 0.55)))
        .rounding(Rounding::same(10.0))
        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(12.5).color(mix(color, Color32::WHITE, 0.45)));
        });
}

/// A progress bar of a real fraction, with its text.
pub fn progress(ui: &mut Ui, fraction: Option<f32>, text: &str, t: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 10.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(5.0), EDGE);
    match fraction {
        Some(f) => {
            let mut filled = rect;
            filled.set_width(rect.width() * f.clamp(0.0, 1.0));
            if filled.width() > 0.5 {
                painter.rect_filled(filled, Rounding::same(5.0), ACCENT);
            }
        }
        // Unknown total: a moving band says "working" without inventing a number.
        None => {
            let w = rect.width() * 0.22;
            let x = rect.left() + (rect.width() - w) * ((t * 0.8).sin() * 0.5 + 0.5);
            painter.rect_filled(Rect::from_min_size(Pos2::new(x, rect.top()), Vec2::new(w, rect.height())), Rounding::same(5.0), VIOLET);
        }
    }
    ui.label(RichText::new(text).small().color(SOFT));
}

/// A row: a label on the left, a coloured status chip on the right.
pub fn status_row(ui: &mut Ui, label: &str, detail: &str, chip_text: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_max_width(ui.available_width() - 170.0);
            ui.label(RichText::new(label).strong().color(Color32::WHITE));
            if !detail.is_empty() {
                ui.label(RichText::new(detail).size(12.5).color(SOFT));
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| chip(ui, chip_text, color));
    });
    ui.add_space(4.0);
}

pub const GOOD: Color32 = Color32::from_rgb(52, 199, 120);
pub const WARN: Color32 = Color32::from_rgb(240, 170, 60);

pub fn title(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).heading().strong().color(Color32::WHITE));
    ui.add_space(8.0);
}

pub fn card<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::none()
        .fill(CARD)
        .stroke(Stroke::new(1.0_f32, EDGE))
        .rounding(Rounding::same(18.0))
        .inner_margin(egui::Margin::same(18.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            body(ui)
        })
        .inner
}

/// A glow behind a rect, `amount` from 0 to 1.
fn glow(painter: &egui::Painter, rect: Rect, rounding: f32, color: Color32, amount: f32) {
    for (grow, alpha) in [(10.0, 0.06), (6.0, 0.10), (3.0, 0.16)] {
        painter.rect_filled(rect.expand(grow * amount), Rounding::same(rounding + grow), with_alpha(color, alpha * amount));
    }
}

pub fn nav_button(ui: &mut Ui, text: &str, selected: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 44.0), Sense::click());
    let hover = ui.ctx().animate_bool(response.id, response.hovered());
    let on = ui.ctx().animate_bool(response.id.with("sel"), selected);
    let painter = ui.painter();
    if on > 0.0 {
        glow(painter, rect, 12.0, ACCENT, on);
    }
    let fill = mix(mix(PANEL, CARD_HOVER, hover), mix(ACCENT, Color32::from_rgb(255, 110, 150), hover), on);
    painter.rect_filled(rect, Rounding::same(12.0), fill);
    painter.rect_filled(
        Rect::from_min_size(rect.left_top() + Vec2::new(0.0, 10.0), Vec2::new(4.0 * (hover.max(on)), 24.0)),
        Rounding::same(2.0),
        SUN,
    );
    painter.text(
        rect.left_center() + Vec2::new(18.0 + 4.0 * hover, 0.0),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(16.0),
        mix(SOFT, Color32::WHITE, hover.max(on)),
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn big_button(ui: &mut Ui, text: &str, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(190.0, 64.0), Sense::click());
    let hover = ui.ctx().animate_bool(response.id, response.hovered());
    let press = if response.is_pointer_button_down_on() { 1.0 } else { 0.0 };
    let rect = rect.shrink(2.0 * press);
    let painter = ui.painter();
    glow(painter, rect, 20.0, color, 0.45 + 0.55 * hover);
    let inner = rect;
    painter.rect_filled(inner, Rounding::same(20.0), mix(color, SUN, 0.25 * hover));
    painter.rect_stroke(inner, Rounding::same(20.0), Stroke::new(1.5_f32, with_alpha(Color32::WHITE, 0.25 + 0.35 * hover)));
    painter.text(inner.center(), Align2::CENTER_CENTER, text, FontId::proportional(24.0 + 1.5 * hover), Color32::WHITE);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn pill(ui: &mut Ui, text: &str, selected: bool) -> Response {
    let galley = ui.painter().layout_no_wrap(text.to_string(), FontId::proportional(15.0), Color32::WHITE);
    let size = galley.size() + Vec2::new(36.0, 18.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hover = ui.ctx().animate_bool(response.id, response.hovered());
    let painter = ui.painter();
    if hover > 0.0 || selected {
        glow(painter, rect, size.y / 2.0, VIOLET, hover.max(if selected { 1.0 } else { 0.0 }));
    }
    painter.rect_filled(rect, Rounding::same(size.y / 2.0), mix(CARD_HOVER, VIOLET, hover.max(if selected { 1.0 } else { 0.0 })));
    painter.rect_stroke(rect, Rounding::same(size.y / 2.0), Stroke::new(1.0_f32, mix(EDGE, Color32::WHITE, hover * 0.5)));
    painter.galley(rect.center() - galley.size() / 2.0, galley, Color32::WHITE);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn preset_card(ui: &mut Ui, label: &str, blurb: &str, selected: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(222.0, 112.0), Sense::click());
    let hover = ui.ctx().animate_bool(response.id, response.hovered());
    let on = ui.ctx().animate_bool(response.id.with("sel"), selected);
    let lift = -3.0 * hover;
    let rect = rect.translate(Vec2::new(0.0, lift));
    let painter = ui.painter();
    glow(painter, rect, 18.0, if selected { ACCENT } else { VIOLET }, hover.max(on) * 0.9);
    painter.rect_filled(rect, Rounding::same(18.0), mix(CARD, CARD_HOVER, hover));
    painter.rect_stroke(rect, Rounding::same(18.0), Stroke::new(1.0 + on, mix(EDGE, ACCENT, on.max(hover * 0.6))));
    painter.text(rect.left_top() + Vec2::new(16.0, 16.0), Align2::LEFT_TOP, label, FontId::proportional(17.0), Color32::WHITE);
    painter.text(rect.left_top() + Vec2::new(16.0, 44.0), Align2::LEFT_TOP, blurb, FontId::proportional(12.5), SOFT);
    if on > 0.0 {
        painter.circle_filled(rect.right_top() + Vec2::new(-18.0, 18.0), 5.0 * on, SUN);
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn stat(ui: &mut Ui, label: &str, value: &str) {
    egui::Frame::none()
        .fill(CARD_HOVER)
        .rounding(Rounding::same(14.0))
        .inner_margin(egui::Margin::symmetric(16.0, 10.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(label).small().color(MUTED));
                ui.label(RichText::new(value).size(22.0).strong().color(Color32::WHITE));
            });
        });
}

/// A switch drawn as a rounded track with a sliding knob.
pub fn toggle(ui: &mut Ui, on: &mut bool) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(52.0, 28.0), Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let t = ui.ctx().animate_bool(response.id, *on);
    let hover = ui.ctx().animate_bool(response.id.with("h"), response.hovered());
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(14.0), mix(EDGE, ACCENT, t));
    let x = egui::lerp(rect.left() + 14.0..=rect.right() - 14.0, t);
    painter.circle_filled(Pos2::new(x, rect.center().y), 10.0 + hover, Color32::WHITE);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn toggle_row(ui: &mut Ui, label: &str, help: &str, on: &mut bool) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(label).strong().color(Color32::WHITE));
            ui.label(RichText::new(help).color(SOFT));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed = toggle(ui, on).changed();
        });
    });
    ui.add_space(6.0);
    changed
}

/// A path text field; true when it was changed and left.
pub fn path_field(ui: &mut Ui, value: &mut String) -> bool {
    let response = ui.add(
        egui::TextEdit::singleline(value)
            .desired_width(f32::INFINITY)
            .margin(Vec2::new(12.0, 8.0))
            .hint_text("C:\\..."),
    );
    response.lost_focus() || (response.changed() && !response.has_focus())
}
