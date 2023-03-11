/* components:
Ui
    gets pen/touch/mouse inputs and sends them to the canvas for drawing or something
Canvas
    responsible for creating and keeping track of drawn strokes
    maybe responsible for displaying them? Or maybe that should should be done by some other component
*/
// @todo can replace some uses of #[cfg(debug_assertions)] with more fine-grained switches, like one
// specifically for showing bounding boxes

use macroquad::prelude::*;
mod canvas;
use canvas::{Canvas, CurveInProgress, Point};

#[macroquad::main("test window")] // window name
async fn main() {

    let mut dbg_string = String::new(); // some debug info shown in a corner of the canvas
    let mut canvas = Canvas::default();
    let mut current_curve: Option<CurveInProgress> = None; // the curve the user is currently drawing

    loop {
        clear_background(BLACK);
        dbg_string.clear();

        let mouse_pos: Point = mouse_position().into();
        dbg_string.push_str(format!(" {},{}", mouse_pos.x, mouse_pos.y).as_str());

        if is_mouse_button_down(MouseButton::Left) {
            dbg_string.push_str(" LEFT");

            if let Some(ref mut curve) = current_curve { curve.add_point(mouse_pos); }
            else {
                current_curve = Some(CurveInProgress::start(mouse_pos));
            }
        }
        else if is_mouse_button_released(MouseButton::Left) { // was it realeased this frame?
            if current_curve.is_some() {
                let finished_curve = current_curve.take().unwrap().finish();
                canvas.add_curve(finished_curve);
            }
        }
        // fun debug output, not actually using this
        if is_mouse_button_down(MouseButton::Right) { dbg_string.push_str(" RIGHT"); }

        // render
        canvas.render();
        // also render the curve the user is currently drawing
        if let Some(ref curve) = current_curve { Canvas::render_curve(&curve.curve); }
        // render debug text
        draw_text(dbg_string.as_str(), 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}
