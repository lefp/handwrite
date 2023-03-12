use macroquad::prelude::*;
use itertools::Itertools;
use geo::{
    self,
    Coord,
    LineString, Intersects, Line,
};

pub type Point = Coord<f32>;
/* @todo "a closed LineString must not self-intersect. Note that its validity is not enforced, and operations
and predicates are undefined on invalid LineStrings".
A user could totally draw a closed self-intersecting curve. How do we ensure the library doesn't bug out as a
result?
*/
type Curve = LineString<f32>;

/// Error returned when starting a new stroke when one is already in progress.
#[derive(Debug)]
pub struct AlreadyInProgress;
/// Error returned when adding a point to or finishing the current stroke,
/// but the current stroke doesn't exist.
#[derive(Debug)]
pub struct NotInProgress;

#[derive(Default)]
pub struct Canvas {
    curves: Vec<Curve>,
    current_curve: Option<Vec<Point>>, // the stroke currently being drawn
    // when erasing, uses `prev_erase_point` to find the line along which the eraser moved between frames
    prev_erase_point: Option<Point>,
}
impl Canvas {
    /// Render a single curve.
    /// This can be used to render curves that aren't strictly part of the canvas yet, such as a stroke that
    /// the user is in the process of drawing.
    fn render_curve(curve: &Vec<Point>) {
        for endpoints in curve.into_iter().tuple_windows() {
            let (p1, p2) = endpoints;
            draw_line(p1.x, p1.y, p2.x, p2.y, 3f32, BLUE);
        }
    }
    
    /// Render all objects on the canvas to the screen.
    pub fn render(&self) {
        for curve in &self.curves { Self::render_curve(&curve.0); }
        if let Some(ref curve) = self.current_curve { Self::render_curve(curve); }
    }

    /// Start drawing a stroke on the canvas.
    /// Returns an error if there is already a stroke in progress.
    pub fn begin_stroke(&mut self, first_point: Point) -> Result<(), AlreadyInProgress> {
        if self.current_curve.is_some() { Err(AlreadyInProgress) }
        else {
            self.current_curve = Some(vec![first_point]);
            Ok(())
        }
    }
    /// Add points to the stroke currently being drawn on the canvas.
    /// Returns an error if there is no stroke in progress.
    pub fn continue_stroke(&mut self, p: Point) -> Result<(), NotInProgress> {
        if let Some(ref mut curve) = self.current_curve {
            curve.push(p);
            Ok(())
        }
        else { Err(NotInProgress) }
    }
    /// Finish drawing the current stroke on the canvas.
    /// This commits the curve to the canvas; you can't add any points to it after this.
    /// If the stroke contains one point or less, simply discards the stroke.
    /// Returns an error if there is no stroke in progress.
    pub fn end_stroke(&mut self) -> Result<(), NotInProgress> {
        if let Some(curve) = self.current_curve.take() {
            // Only commit the curve if it has more than one point; otherwise just discard it, because the doc
            // says "a LineString is valid if it is either empty or contains 2 or more coordinates."
            if curve.len() > 1 { self.curves.push(LineString::from(curve)); }
            Ok(())
        }
        else { Err(NotInProgress) }
    }
    /// Is a stroke currently being drawn on the canvas?
    /// True iff the latest stroke created by `begin_stroke` hasn't yet been ended via `end_stroke`.
    pub fn is_stroke_in_progress(&self) -> bool { self.current_curve.is_some() }

    /// Delete every curve that intersects the line segment [p1, p2].
    /// Note that the eraser is considered a point; it has no thickness.
    fn erase(&mut self, p1: Point, p2: Point) {
        let erase_line = Line::new(p1, p2);

        let mut i = 0;
        while i < self.curves.len() {
            if self.curves[i].intersects(&erase_line) { self.curves.swap_remove(i); } else { i += 1; }
        }
    }

    pub fn begin_erasure(&mut self, first_point: Point) -> Result<(), AlreadyInProgress> {
        if self.prev_erase_point.is_some() { Err(AlreadyInProgress) }
        else {
            self.prev_erase_point = Some(first_point);
            Ok(())
        }
    }
    pub fn continue_erasure(&mut self, p: Point) -> Result<(), NotInProgress> {
        if let Some(prev) = self.prev_erase_point {
            self.erase(prev, p);
            self.prev_erase_point = Some(p);
            Ok(())
        }
        else { Err(NotInProgress) }
    }
    pub fn end_erasure(&mut self) -> Result<(), NotInProgress> {
        // set to None, and error if it was already None
        if self.prev_erase_point.take().is_some() { Ok(()) } else { Err(NotInProgress) }
    }
    pub fn is_erasure_in_progress(&self) -> bool { self.prev_erase_point.is_some() }
}
