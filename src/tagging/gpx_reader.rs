use std::io::BufReader;

use gpx::read;
use gpx::Gpx;

#[derive(Clone)]
pub struct GpxPoint {
    pub lat: f64,
    pub lon: f64,
    pub elevation: Option<f64>,
    pub time: Option<gpx::Time>,
    pub name: Option<String>,
    pub success: bool,
}

// We cast a wide net for common failure terms in waypoint type to determine if a waypoint is successful or not, because there is no standard for this in GPX.
fn waypoint_is_successful(wpt: &gpx::Waypoint) -> bool {
    let failure_terms = ["fail", "error", "busy", "abort", "miss"];
    let wpt_type = wpt.type_.clone();
    if let Some(wpt_type) = wpt_type {
        let wpt_type_lower = wpt_type.to_lowercase();
        !failure_terms.iter().any(|term| wpt_type_lower.contains(term))
    } else {
        // No type. Assume success
        true
    }
}

pub fn read_gpx(data: &[u8]) -> Result<Vec<GpxPoint>, Box<dyn std::error::Error>> {
    let reader = BufReader::new(data);
    let gpx: Gpx = read(reader)?;

    let wpts = gpx.waypoints;
    let tracks = gpx.tracks;

    // We do not need to interpolate position with track segments if there are waypoints in the GPX file.
    if !wpts.is_empty() {
        let points = wpts.into_iter().map(|wpt| GpxPoint {
            lat: wpt.point().y(),
            lon: wpt.point().x(),
            elevation: wpt.elevation,
            time: wpt.time,
            name: wpt.name.clone(),
            success: waypoint_is_successful(&wpt),
        }).collect();
        return Ok(points);
    }

    // Return error
    Err("No waypoints found in GPX file".into())
}
