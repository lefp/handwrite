use macroquad::prelude::*;
use glam::Vec2;

pub type Point = Vec2;

/// A rectangle with no rotation information.
/// Satisfies p1.x <= p2.x, p1.y <= p2.y
pub struct Box {
    p1: Point,
    p2: Point,
}
impl Box {
    /// Expands exactly as much as needed to contain `p`.
    /// The new region always contains the old one; interior points are never lost.
    pub fn expand_to_contain(&mut self, p: Point) {
        self.p1 = self.p1.min(p);
        self.p2 = self.p2.max(p);
    }

    /// Returns true iff `p` is in the box (boundary included).
    pub fn contains(&self, p: Point) -> bool {
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
pub struct Rectangle {
    a: Point,
    ab: Vec2,
    ad: Vec2,
}
impl Rectangle {
    /// Is the point inside the rectangle?
    /// Includes the boundary.
    pub fn contains(&self, p: Point) -> bool {
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
    pub fn along_line_segment(p: Point, v: Vec2, w: f32) -> Self {
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
pub struct Curve {
    points: Vec<Point>,
    bounding_box: Box,
}
impl Curve {
    pub fn new(first_point: Point) -> Self {
        Self {
            points: vec![first_point],
            bounding_box: Box { p1: first_point, p2: first_point },
        }
    }

    pub fn add_point(&mut self, p: Point) {
        self.points.push(p);
        // update the bounding box
        self.bounding_box.expand_to_contain(p);
    }

    // /// Return the new set of curves that results from erasing every point in `region`
    // fn erase(region: Box) {
        
    // }
}

/// Error returned when starting a new stroke when one is already in progress.
#[derive(Debug)]
pub struct AlreadyExists;
/// Error returned when adding a point to or finishing the current stroke,
/// but the current stroke doesn't exist.
#[derive(Debug)]
pub struct DoesntExist;

#[derive(Default)]
pub struct Canvas {
    curves: Vec<Curve>,
    current_curve: Option<Curve>,
}
impl Canvas {
    /// Render a single curve.
    /// This can be used to render curves that aren't strictly part of the canvas yet, such as a stroke that
    /// the user is in the process of drawing.
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
        if let Some(ref curve) = self.current_curve { Self::render_curve(curve); }
    }

    /// Start drawing a stroke on the canvas.
    /// Returns an error if there is already a stroke in progress.
    pub fn begin_stroke(&mut self, first_point: Point) -> Result<(), AlreadyExists> {
        if self.current_curve.is_some() { Err(AlreadyExists) }
        else {
            self.current_curve = Some(Curve::new(first_point));
            Ok(())
        }
    }
    /// Add points to the stroke currently being drawn on the canvas.
    /// Returns an error if there is no stroke in progress.
    pub fn continue_stroke(&mut self, p: Point) -> Result<(), DoesntExist> {
        if let Some(ref mut curve) = self.current_curve {
            curve.add_point(p);
            Ok(())
        }
        else { Err(DoesntExist) }
    }
    /// Finish drawing the current stroke on the canvas.
    /// This commits the curve to the canvas; you can't add any points to it after this.
    /// Returns an error if there is no stroke in progress.
    pub fn end_stroke(&mut self) -> Result<(), DoesntExist> {
        if let Some(curve) = self.current_curve.take() {
            self.curves.push(curve);
            Ok(())
        }
        else { Err(DoesntExist) }
    }
    /// Is a stroke currently being drawn on the canvas?
    /// True iff the latest stroke created by `begin_stroke` hasn't yet been ended via `end_stroke`.
    pub fn is_stroke_in_progress(&self) -> bool { self.current_curve.is_some() }
}
