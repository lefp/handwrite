/* components:
Ui
    gets pen/touch/mouse inputs and sends them to the canvas for drawing or something
Canvas
    responsible for creating and keeping track of drawn strokes
    maybe responsible for displaying them? Or maybe that should should be done by some other component
*/

use macroquad::prelude::*;

#[derive(Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}
impl Point {
    fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
}

// // a sequence of points. No thickness information attached!
// struct Curve {
//     points: Vec<Point>
// }
// impl Curve {
//     fn new(first_point: Point) -> Self {
//         Self { points: vec![first_point] }
//     }
//     fn add_point(&mut self, p: Point) {
//         self.points.push(p);
//     }
// }

type Curve = Vec<Point>;

#[derive(Default)]
struct Canvas {
    curves: Vec<Curve>,
    current_curve: Option<Curve>, // curve currently being drawn
}
impl Canvas {
    /// Begin drawing a curve.
    /// Every subsequent call to `continue_curve` adds a point to this curve.
    pub fn start_curve(&mut self, first_point: Point) {
        assert!(
            self.current_curve.is_none(),
            "Attempted to start a new curve before ending the previous one"
        );
        self.current_curve = Some(vec![first_point]);
    }
    /// Add a point to the curve currently being drawn.
    pub fn continue_curve(&mut self, new_point: Point) {
        self.current_curve.as_mut().expect("Attempted to continue a curve that doesn't exist")
            .push(new_point);
    }
    /// Stop drawing the current curve.
    /// It is illegal to subsequently call `continue_curve` or `end_curve` before `start_curve`.
    pub fn end_curve(&mut self) {
        self.curves.push(
            self.current_curve.take().expect("Attempted to end a curve that doesn't exist")
        );
    }

    /// Render a single curve.
    fn render_curve(curve: &Curve) {
        for endpoints in curve.windows(2) {
            let p1 = endpoints[0];
            let p2 = endpoints[1];
            draw_line(p1.x, p1.y, p2.x, p2.y, 3f32, BLUE);
        }
    }
    
    /// Render all objects on the canvas to the screen.
    pub fn render(&self) {
        for curve in &self.curves { Self::render_curve(curve); }
        if let Some(curve) = &self.current_curve { Self::render_curve(curve); }
    }
}

#[macroquad::main("test window")] // window name
async fn main() {

    let mut dbg_string = String::new();
    let mut canvas = Canvas::default();

    let mut lmb_was_already_pressed = false;

    loop {
        clear_background(BLACK);
        dbg_string.clear();

        let (mouse_x, mouse_y) = mouse_position();
        dbg_string.push_str(format!("{mouse_x},{mouse_y}").as_str());

        // @todo instead of checking if the button was already pressed, maybe shoud just check if a curve was
        // already being drawn.
        if is_mouse_button_down(MouseButton::Left) {
            dbg_string.push_str(" LEFT");

            let point = Point::new(mouse_x, mouse_y);
            if lmb_was_already_pressed { canvas.continue_curve(point); }
            else {
                canvas.start_curve(point);
                lmb_was_already_pressed = true;
            }
        }
        else if is_mouse_button_released(MouseButton::Left) { // was it realeased this frame?
            canvas.end_curve();
            lmb_was_already_pressed = false;
        }
        // fun debug output, not actually using this
        if is_mouse_button_down(MouseButton::Right) { dbg_string.push_str(" RIGHT"); }

        // render
        canvas.render();
        draw_text(dbg_string.as_str(), 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}
