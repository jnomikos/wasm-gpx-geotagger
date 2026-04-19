use dioxus::prelude::*;
use dioxus_leaflet::{Map, MapPosition, Marker, MarkerIcon, Popup, LatLng};

use crate::tagging::GpxPoint;

const FAIL_PIN: Asset  = asset!("/assets/fail-pin.svg");

#[component]
pub fn SampleMap(gpx_points_signal: Signal<Option<Vec<GpxPoint>>>) -> Element {
    use_effect(|| {
        let _ = document::eval(r#"
            setTimeout(function() {
                var container = document.querySelector('.leaflet-container');
                if (container && container._leaflet_map) {
                    container._leaflet_map.invalidateSize();
                }
            }, 100);
        "#);
    });

    use_effect(move || {
        if let Some(points) = gpx_points_signal.read().as_ref() {
            // Loop through all points to find the bounds of the map and the center point
            let mut min_lat = std::f64::MAX;
            let mut max_lat = std::f64::MIN;
            let mut min_lon = std::f64::MAX;
            let mut max_lon = std::f64::MIN;
            
            for point in points {
                if point.lat < min_lat {
                    min_lat = point.lat;
                }
                if point.lat > max_lat {
                    max_lat = point.lat;
                }
                if point.lon < min_lon {
                    min_lon = point.lon;
                }
                if point.lon > max_lon {
                    max_lon = point.lon;
                }
            }

            let center_lat = (min_lat + max_lat) / 2.0;
            let center_lon = (min_lon + max_lon) / 2.0;
            // Set zoom level based on bounds
            let lat_diff = max_lat - min_lat;
            let lon_diff = max_lon - min_lon;

            document::eval(&format!(r#"
            (async () => {{
                var scriptTag = document.querySelector('script[src*="dioxus_leaflet"]');
                if (!scriptTag) {{ console.error('dioxus_leaflet script tag not found'); return; }}
                var el = document.querySelector('[id^="dioxus-leaflet-map-"]');
                if (!el) {{ console.error('dioxus-leaflet map element not found'); return; }}
                
                var mapId = parseInt(el.id.replace('dioxus-leaflet-map-', ''));
                var module = await import(scriptTag.src);
                var map = await module.get_map(mapId);
                
                map.fitBounds([
                    [{min_lat}, {min_lon}], // SouthWest corner
                    [{max_lat}, {max_lon}]  // NorthEast corner
                ], {{ 
                    padding: [40, 40], 
                    maxZoom: 20,
                    animate: true
                }});
            }})();
        "#));
        }
    });

    rsx! {
        div {
            id: "map",
            visibility: gpx_points_signal.read().is_some().then(|| "visible").unwrap_or("hidden"),
            Map {
                initial_position: MapPosition::new(50.505, -0.09, 5.0),
                for signal in gpx_points_signal.read().as_ref() {
                    for (i, point) in signal.iter().enumerate() {
                        if point.success {
                            Marker {
                                key: "marker-{i}-{point.lat}-{point.lon}",
                                coordinate: LatLng::new(point.lat, point.lon),
                                Popup {
                                    if let Some(name) = &point.name {
                                        b { "{name}" }
                                        br { }
                                    }
                                    "Lat: {point.lat}, Lon: {point.lon}"
                                }
                            }
                        } else {
                            Marker {
                                key: "marker-{i}-{point.lat}-{point.lon}",
                                coordinate: LatLng::new(point.lat, point.lon),
                                icon: MarkerIcon {
                                    icon_url: FAIL_PIN.to_string(),
                                    icon_size: Some((32, 32)),
                                    icon_anchor: Some((16, 32)),
                                    popup_anchor: None,
                                    shadow_url: None,
                                    shadow_size: None,
                                },
                                Popup {
                                    if let Some(name) = &point.name {
                                        b { "{name} (Failed)" }
                                        br { }
                                    }
                                    "Lat: {point.lat}, Lon: {point.lon}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}