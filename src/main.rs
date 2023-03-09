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

#[derive(Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}
impl Point {
    fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }

    /// Component-wise min.
    fn min(&self, p: Point) -> Point {
        Point { x: self.x.min(p.x), y: self.y.min(p.y) }
    }
    /// Component-wise max.
    fn max(&self, p: Point) -> Point {
        Point { x: self.x.max(p.x), y: self.y.max(p.y) }
    }
}
impl From<(f32, f32)> for Point {
    fn from(point: (f32, f32)) -> Self {
        Self::new(point.0, point.1)
    }
}
impl Into<(f32, f32)> for Point {
    fn into(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

/// A rectangle with no rotation information.
/// Satisfies p1.x <= p2.x, p1.y <= p2.y
struct Box {
    p1: Point,
    p2: Point,
}
impl Box {
    /// Expands exactly as much as needed to contain `p`.
    /// The new region always contains the old one; interior points are never lost.
    fn expand_to_contain(&mut self, p: Point) {
        self.p1 = self.p1.min(p);
        self.p2 = self.p2.max(p);
    }
}

/// A sequence of points. No thickness information attached!
/// The bounding box is provided to easily find curves intersecting a region; you can first check whether the
/// bounding box intersect the region before checking every point on the curve, which allows you to quickly
/// discard curves that are not near the region.
/* @todo would it be more efficient to compute a bounding rectangle of minimal area? E.g. a diagonal line
segment is much more precisely constrained by a rotated rectangle than a horizontal rectangle. This would
reduce the number of curves whose bounding box intersect the index region. Some questions:
- Is this efficient to compute?
- Is this efficient to compute iteratively? (i.e. every time a new point is added to the curve)
    Although, why would we ever need to compute it iteratively? New points are only added while the curve is
    being drawn.
- Is it easy to compute the intersection of a regular box with a rotated rectangle?
*/
struct Curve {
    points: Vec<Point>,
    bounding_box: Box,
}
impl Curve {
    fn new(first_point: Point) -> Self {
        Self {
            points: vec![first_point],
            bounding_box: Box { p1: first_point, p2: first_point },
        }
    }
    fn add_point(&mut self, p: Point) {
        self.points.push(p);
        // update the bounding box
        self.bounding_box.expand_to_contain(p);
    }
    // @todo fn erase()
}

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
        let curve = Curve::new(first_point);
        self.current_curve = Some(curve);
    }
    /// Add a point to the curve currently being drawn.
    pub fn continue_curve(&mut self, new_point: Point) {
        self.current_curve.as_mut().expect("Attempted to continue a curve that doesn't exist")
            .add_point(new_point);
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
        for endpoints in curve.points.windows(2) {
            let p1 = endpoints[0];
            let p2 = endpoints[1];
            draw_line(p1.x, p1.y, p2.x, p2.y, 3f32, BLUE);
        }

        // draw the bounding box
        #[cfg(debug_assertions)]
        let (x1, y1) = curve.bounding_box.p1.into();
        let (x2, y2) = curve.bounding_box.p2.into();
        //
        draw_rectangle_lines(x1, y1, x2 - x1, y2 - y1, 2f32, RED);
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
