//! Scrollbars share the canvas view and its rotation, rather than maintaining
//! a second scroll position around an independently panned document.
use super::*;

const WIDTH: f32 = 16.0;

pub(super) fn viewport(full: Rect, enabled: bool) -> Rect {
    if enabled { Rect::from_min_max(full.min, (full.max - vec2(WIDTH, WIDTH)).max(full.min)) } else { full }
}

#[derive(Clone, Copy)]
struct Axis {
    min: f64,
    max: f64,
    at: f64,
    page: f64,
}
impl Axis {
    fn new(min: f64, max: f64, at: f64, page: f64) -> Self {
        Self { min: min.min(at), max: (max - page).max(at), at, page }
    }
}

fn bar(ui: &mut egui::Ui, rect: Rect, axis: Axis, vertical: bool, language: &str, pane: u8) -> Option<f64> {
    if !rect.is_positive() {
        return None;
    }
    let id = ui.id().with((if vertical { "canvas-scroll-y" } else { "canvas-scroll-x" }, pane));
    let t = Tokens::get(ui.ctx());
    let length = if vertical { rect.height() } else { rect.width() };
    let span = axis.max - axis.min;
    let enabled = span.is_finite() && span > 0.0 && axis.page.is_finite() && axis.page > 0.0;
    let thumb_length = if enabled { (length * (axis.page / (span + axis.page)) as f32).clamp(WIDTH.min(length), length) } else { length };
    let travel = (length - thumb_length).max(0.0);
    let fraction = if enabled { ((axis.at - axis.min) / span).clamp(0.0, 1.0) as f32 } else { 0.0 };
    let start = if vertical { rect.top() } else { rect.left() };
    let pos = start + fraction * travel;
    let thumb = if vertical {
        Rect::from_min_size(pos2(rect.left() + 2.0, pos), vec2((rect.width() - 4.0).max(0.0), thumb_length))
    } else {
        Rect::from_min_size(pos2(pos, rect.top() + 2.0), vec2(thumb_length, (rect.height() - 4.0).max(0.0)))
    };
    let response = ui.interact(rect, id, if enabled { Sense::click_and_drag() } else { Sense::hover() });
    response.widget_info(|| egui::WidgetInfo::slider(enabled, axis.at, crate::i18n::tr(language, if vertical { "Vertical" } else { "Horizontal" })));
    ui.painter().rect_filled(rect, 0.0, t.panel_darker);
    ui.painter().rect_filled(thumb, 3.0, if response.hovered() || response.dragged() { t.text_dim } else { t.border });
    if !enabled {
        return None;
    }
    let coordinate = |p: Pos2| if vertical { p.y } else { p.x };
    let mut next = axis.at;
    if response.clicked()
        && let Some(p) = response.interact_pointer_pos()
    {
        response.request_focus();
        if coordinate(p) < pos {
            next -= axis.page;
        } else if coordinate(p) > pos + thumb_length {
            next += axis.page;
        }
    }
    if response.drag_started() {
        response.request_focus();
        let pressed = ui.input(|i| i.pointer.press_origin()).unwrap_or(thumb.center());
        let grab = if thumb.contains(pressed) { coordinate(pressed) - pos } else { thumb_length / 2.0 };
        ui.data_mut(|data| data.insert_temp(id.with("grab"), grab));
    }
    if response.dragged()
        && travel > 0.0
        && let Some(pointer) = response.interact_pointer_pos()
    {
        let grab = ui.data(|data| data.get_temp::<f32>(id.with("grab"))).unwrap_or(thumb_length / 2.0);
        next = axis.min + f64::from(((coordinate(pointer) - grab - start) / travel).clamp(0.0, 1.0)) * span;
    }
    if response.has_focus() {
        let arrows = ui.input(|input| {
            [egui::Key::ArrowLeft, egui::Key::ArrowRight, egui::Key::ArrowUp, egui::Key::ArrowDown].iter().any(|key| input.key_pressed(*key))
        });
        ui.memory_mut(|memory| {
            if arrows {
                memory.move_focus(egui::FocusDirection::None);
            }
            memory.set_focus_lock_filter(response.id, egui::EventFilter { horizontal_arrows: true, vertical_arrows: true, ..Default::default() });
        });
        ui.input_mut(|input| {
            use egui::{Key, Modifiers};
            for (key, delta) in [
                (Key::ArrowLeft, -axis.page / 10.0),
                (Key::ArrowUp, -axis.page / 10.0),
                (Key::ArrowRight, axis.page / 10.0),
                (Key::ArrowDown, axis.page / 10.0),
                (Key::PageUp, -axis.page),
                (Key::PageDown, axis.page),
            ] {
                if input.consume_key(Modifiers::NONE, key) {
                    next += delta;
                }
            }
            if input.consume_key(Modifiers::NONE, Key::Home) {
                next = axis.min;
            }
            if input.consume_key(Modifiers::NONE, Key::End) {
                next = axis.max;
            }
        });
    }
    next = next.clamp(axis.min, axis.max);
    (next != axis.at).then_some(next)
}

pub(super) fn show(app: &mut DesignApp, ui: &mut egui::Ui, full: Rect, screen: Rect, rect: Rect) {
    let Some(view) = app.view().copied() else { return };
    if !view.zoom.is_finite() || view.zoom <= 0.0 || !screen.is_positive() {
        return;
    }
    let Some(state) = app.session.active() else { return };
    let layout = CanvasLayout::new(&state.doc, state.editing_parents);
    let turn = Affine::rotate(f64::from(view.rotation % 4) * std::f64::consts::FRAC_PI_2);
    let bounds = turn.transform_rect_bbox(layout.pasteboard);
    let center =
        turn * Point::new(view.origin.x + f64::from(rect.width()) / (2.0 * view.zoom), view.origin.y + f64::from(rect.height()) / (2.0 * view.zoom));
    let (w, h) = (f64::from(screen.width()) / view.zoom, f64::from(screen.height()) / view.zoom);
    let (x, y) = (center.x - w / 2.0, center.y - h / 2.0);
    if ![bounds.x0, bounds.y0, bounds.x1, bounds.y1, x, y, w, h].iter().all(|v| v.is_finite()) {
        return;
    }
    let end = viewport(full, true).max;
    let rx = Rect::from_min_max(pos2(screen.left(), end.y), pos2(end.x, full.bottom()));
    let ry = Rect::from_min_max(pos2(end.x, screen.top()), pos2(full.right(), end.y));
    let nx = bar(ui, rx, Axis::new(bounds.x0, bounds.x1, x, w), false, &app.ui.language, app.pane);
    let ny = bar(ui, ry, Axis::new(bounds.y0, bounds.y1, y, h), true, &app.ui.language, app.pane);
    if nx.is_some() || ny.is_some() {
        let center = turn.inverse() * Point::new(nx.unwrap_or(x) + w / 2.0, ny.unwrap_or(y) + h / 2.0);
        let origin = Point::new(center.x - f64::from(rect.width()) / (2.0 * view.zoom), center.y - f64::from(rect.height()) / (2.0 * view.zoom));
        let _ = app.run("view.pan", json!({"x":origin.x,"y":origin.y}));
        ui.ctx().request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    #[test]
    fn thumb_drag_preserves_the_grab_point_and_keyboard_reaches_the_limits() {
        let mut h = Harness::builder().with_size(vec2(400.0, 100.0)).build_ui_state(
            |ui, at: &mut f64| {
                if let Some(next) =
                    bar(ui, Rect::from_min_size(pos2(20.0, 20.0), vec2(300.0, 16.0)), Axis::new(0.0, 1000.0, *at, 100.0), false, "", 0)
                {
                    *at = next;
                }
            },
            400.0,
        );
        h.run_steps(3);
        // 30 px thumb at 120 px along the track: grab its centre.
        let from = pos2(155.0, 28.0);
        h.event(egui::Event::PointerMoved(from));
        h.run_steps(1);
        h.event(egui::Event::PointerButton { pos: from, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
        h.run_steps(1);
        h.event(egui::Event::PointerMoved(from + vec2(30.0, 0.0)));
        h.run_steps(2);
        assert!((*h.state() - 500.0).abs() < 0.01, "drag must not jump: {}", h.state());
        h.event(egui::Event::PointerButton {
            pos: from + vec2(30.0, 0.0),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        });
        h.run_steps(2);
        h.key_press(egui::Key::End);
        h.run_steps(2);
        assert_eq!(*h.state(), 900.0);
        h.key_press(egui::Key::Home);
        h.run_steps(2);
        assert_eq!(*h.state(), 0.0);
        assert_eq!(h.get_by_label("Horizontal").rect().width(), 300.0);
    }
}
