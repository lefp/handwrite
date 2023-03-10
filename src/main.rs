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
use glam::Vec2;

type Point = Vec2;

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

    /// Returns true iff `p` is in the box (boundary included).
    fn contains(&self, p: Point) -> bool {
        self.p1.x <= p.x &&
        self.p1.y <= p.y &&
        self.p2.x >= p.x &&
        self.p2.y >= p.y
    }
}

/// A rectangle with an orientation.
/// Think of it as the rectangle formed by connecting points `abcda` in that order.
/// `a` defines the position of one corner.
/// `ab` and `ad` are an orthogonal pair of vectors defining the sides of the rectangle adjacent to a.
// @todo determine whether the sides should be allowed to be 0 or non-orthogonal
struct Rectangle {
    a: Point,
    ab: Vec2,
    ad: Vec2,
}
impl Rectangle {
    /// Is the point inside the rectangle?
    /// Includes the boundary.
    fn contains(&self, p: Point) -> bool {
        let mut contains = true;

        // `p` is in the rectangle iff the projection of `ap` onto the basis `{ab, ad}` is in the projected
        // rectangle `{z | 0 <= z_ab <= ab_ab, 0 <= z_ad <= ad_ad}`, where `z_ab` represents to component of
        // `z` in the direction `ab`.
        let ap: Vec2 = p - self.a;
        for basis_vec in [self.ab, self.ad] {
            let proj = ap.dot(basis_vec);
            // @todo is this the right operator for boolean AND?
            contains &= 0. <= proj && proj <= basis_vec.dot(basis_vec);
        }

        contains
    }

    /// Creates a rectangle along the line segment `[p, p+v]`.
    /// `w` is the width of the rectangle.
    /**
          ---------------------       ---
          |                   |        |
        p *-------------------> v      w
          |                   |        |
          ---------------------       ---
    */
    // @todo probably needs a unit test
    fn along_line_segment(p: Point, v: Vec2, w: f32) -> Self {
        let ab = v;
        let ad = v.perp().normalize_or_zero() * w;
        let a = p - 0.5 * ad;
        Self { a, ab, ad }
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

    // /// Return the new set of curves that results from erasing every point in `region`
    // fn erase(region: Box) {
        
    // }
}

/// The interface for incrementally creating a curve (e.g. while the user is drawing).
/// Consumes itself when you call `finish` to ensure you don't accidentally add points to it later.
struct CurveInProgress {
    curve: Curve,
}
impl CurveInProgress {
    /// Begin drawing a curve.
    pub fn start(first_point: Point) -> Self {
        Self { curve: Curve::new(first_point) }
    }
    /// Add a new point to the curve.
    pub fn add_point(&mut self, p: Point) {
        self.curve.add_point(p);
    }
    /// Finish the current curve.
    /// Returns the finished curve.
    pub fn finish(self) -> Curve {
        self.curve
    }
}

#[derive(Default)]
struct Canvas {
    curves: Vec<Curve>,
}
impl Canvas {
    /// Render a single curve.
    /// This can be used to render curves that aren't strictly part of the canvas yet, such as a stroke that
    /// the user is in the process of drawing.
    pub fn render_curve(curve: &Curve) {
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
    }

    /// Add a new curve to the canvas.
    pub fn add_curve(&mut self, curve: Curve) {
        self.curves.push(curve);
    }
}

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
