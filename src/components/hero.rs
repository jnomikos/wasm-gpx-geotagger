use dioxus::prelude::*;
use dioxus::html::FileData;
use bytes::Bytes;
use js_sys::Array;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};
use crate::tagging::*;

#[derive(Clone, Default)]
struct TaggingJob {
    active: bool,
    tagged: usize,
    total: usize,
    elapsed: std::time::Duration,
}

fn mime_type(filename: &str) -> &'static str {
    match filename.rsplit('.').next().map(|e| e.to_lowercase()).as_deref() {
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("tiff") | Some("tif") => "image/tiff",
        Some("jxl") => "image/jxl",
        Some("heif") | Some("heic") => "image/heif",
        _ => "image/jpeg",
    }
}

fn trigger_download(filename: &str, data: Bytes) {
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");

    let array = Array::new();
    // Safety: Blob::new_with_u8_array_sequence copies the data synchronously,
    // so the Bytes backing memory is valid for the entire duration of this call.
    let uint8 = unsafe { js_sys::Uint8Array::view(&data) };
    array.push(&uint8);

    let opts = BlobPropertyBag::new();
    opts.set_type(mime_type(filename));

    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &opts)
        .expect("Blob construction failed");
    let url = Url::create_object_url_with_blob(&blob)
        .expect("createObjectURL failed");

    let anchor: HtmlAnchorElement = document
        .create_element("a")
        .expect("createElement failed")
        .dyn_into()
        .expect("dyn_into HtmlAnchorElement failed");

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    Url::revoke_object_url(&url).expect("revokeObjectURL failed");
}

#[component]
pub fn Hero(gpx_points_signal: Signal<Option<Vec<GpxPoint>>>) -> Element {
    let mut selected_gpx_filename = use_signal(|| None::<String>);
    let mut selected_photos = use_signal(|| Vec::<FileData>::new());
    let mut job = use_signal(TaggingJob::default);
    let failed_count = gpx_points_signal.read().as_ref()
        .map(|pts| pts.iter().filter(|p| !p.success).count())
        .unwrap_or(0);
    rsx! {
        div {
            id: "hero",
            div { id: "hero-content",
                h2 {
                    id: "title",
                    "GPX Geotagger"
                }
                p {
                    id: "description",
                    "Insert GPX file and select a directory of photos to geotag with the GPX data."
                }
            }
            div { id: "file-browse",
                input {
                    r#type: "file",
                    accept: ".gpx",
                    id: "gpx-picker",
                    style: "display: none;",
                    multiple: false,
                    onchange: move |evt| {
                        spawn(async move {
                            if let Some(file) = evt.files().into_iter().next() {
                                selected_gpx_filename.set(Some(file.name()));
                                if let Ok(bytes) = file.read_bytes().await {
                                    gpx_points_signal.set(read_gpx(&bytes).ok());
                                }
                            }
                        });
                    }
                }
                button {
                    id: "file-browse-button",
                    onclick: move |_| {
                        document::eval("document.getElementById('gpx-picker').click()");
                    },
                    if let Some(gpx_file) = selected_gpx_filename.read().as_ref() {
                        "{gpx_file}"
                    } else {
                        "Select GPX File"
                    }
                }
                input {
                    r#type: "file",
                    id: "photo-picker",
                    style: "display: none;",
                    "webkitdirectory": "true",
                    onchange: move |evt| {
                        let photos: Vec<FileData> = evt.files()
                            .into_iter()
                            .filter(|f| get_file_extension(&f.name()).is_some())
                            .collect();
                        selected_photos.set(photos);
                    }
                }
                button {
                    id: "file-browse-button",
                    onclick: move |_| {
                        document::eval("document.getElementById('photo-picker').click()");
                    },
                    if !selected_photos.read().is_empty() {
                        "{selected_photos.len()} photo(s) selected"
                    } else {
                        "Select Photos Directory"
                    }
                }

                if failed_count > 0 {
                    p {
                        id: "failure-warning",
                        "⚠️ Warning: GPX file contains {failed_count} failed point(s). Failed points will be skipped during geotagging.",
                    }
                }

                button {
                    id: "geotag-button",
                    disabled: selected_gpx_filename.read().is_none() || selected_photos.read().is_empty() || job.read().active,
                    onclick: move |_| {
                        spawn(async move {
                            if let Some(points) = gpx_points_signal.read().as_ref() {
                                let pairs: Vec<_> = points.iter()
                                    .filter(|p| p.success)
                                    .zip(selected_photos.read().iter())
                                    .map(|(p, f)| (p.clone(), f.clone()))
                                    .collect();

                                job.set(TaggingJob { active: true, tagged: 0, total: pairs.len(), elapsed: std::time::Duration::default() });

                                let t_start = web_sys::window().unwrap().performance().unwrap().now();
                                for (point, photo) in &pairs {
                                    if let Some(file_type) = get_file_extension(&photo.name()) {
                                        if let Ok(bytes) = photo.read_bytes().await {
                                            match tag_image(bytes.to_vec(), point, file_type) {
                                                Ok(tagged) => {
                                                    info!("Tagged {} at ({}, {})", photo.name(), point.lat, point.lon);
                                                    trigger_download(&photo.name(), tagged);
                                                }
                                                Err(e) => error!("Failed to tag {}: {}", photo.name(), e),
                                            }
                                        }
                                    } else {
                                        error!("Unsupported file type: {}", photo.name());
                                    }
                                    job.write().tagged += 1;
                                }

                                let elapsed_ms = web_sys::window().unwrap().performance().unwrap().now() - t_start;
                                job.write().elapsed = std::time::Duration::from_millis(elapsed_ms as u64);
                                job.write().active = false;
                            }
                        });
                    },
                    if job.read().active {
                        "Tagging Images..."
                    } else {
                        "Geotag Photos"
                    }
                }
                if job.read().active {
                    div { id: "progress-bar-track",
                        div {
                            id: "progress-bar-fill",
                            style: "width: {job.read().tagged * 100 / job.read().total.max(1)}%",
                        }
                    }
                    p { id: "progress-label",
                        "{job.read().tagged} / {job.read().total}"
                    }
                }
                if !job.read().active && job.read().elapsed != std::time::Duration::default() {
                    p { id: "completion-label",
                        "Tagged {job.read().total} photo(s) in {job.read().elapsed.as_secs_f32():.1}s"
                    }
                }
                p {
                    id: "file-hint",
                    "Supported: JPEG, PNG, WebP, JXL, TIFF, HEIF"
                    br {}
                    span { id: "file-hint-optimized", "⚡ Optimized: JPEG, PNG, WebP" }
                }
            }
        }
    }
}
