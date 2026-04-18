use dioxus::prelude::*;

#[component]
pub fn Hero() -> Element {
    let mut selected_gpx = use_signal(|| None::<String>);
    let mut selected_photos = use_signal(|| Vec::<String>::new());
    rsx! {
        // We can create elements inside the rsx macro with the element name followed by a block of attributes and children.
        div {
            // Attributes should be defined in the element before any children
            id: "hero",
            div { id: "hero-content",
                h2 { 
                    id: "title",
                    "GPX Geotagger" 
                }
                p {
                    id: "description",
                    "Insert GPX file and select jpeg photos to geotag with the GPX data."
                }
            }
            // After all attributes are defined, we can define child elements and components
            div { id: "file-browse",
                input {
                    r#type: "file",
                    accept: ".gpx",
                    id: "gpx-picker",
                    style: "display: none;",
                    multiple: false,
                    onchange: move |evt| {
                        let files = evt.files();
                        selected_gpx.set(files.into_iter().next().map(|file| file.name()));
                    }
                }
                button {
                    id: "file-browse-button",
                    onclick: move |_| {
                        let mut eval = document::eval(
                            "document.getElementById('gpx-picker').click()"
                        );
                    },
                    if let Some(gpx_file) = selected_gpx.read().as_ref() {
                        "{gpx_file}"
                    } else {
                        "Select GPX File"
                    }
                }
                input {
                    r#type: "file",
                    accept: "image/jpeg",
                    id: "photo-picker",
                    style: "display: none;",
                    multiple: true,
                    onchange: move |evt| {
                        let files = evt.files();
                        selected_photos.set(files.into_iter().map(|file| file.name()).collect());
                    }
                }
                button {
                    id: "file-browse-button",
                    onclick: move |_| {
                        let mut eval = document::eval(
                            "document.getElementById('photo-picker').click()"
                        );
                    },
                    if !selected_photos.read().is_empty() {
                        "{selected_photos.len()} photos selected"
                    } else {
                        "Select Photos"
                    }
                }

                button {
                    id: "geotag-button",
                    disabled: selected_gpx.read().is_none() || selected_photos.read().is_empty(),
                    onclick: move |_| {
                        // Here we would add the logic to geotag the photos with the GPX data
                    },
                    "Geotag Photos"
                }
            }
        }
    }
}
