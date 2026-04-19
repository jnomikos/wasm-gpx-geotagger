This is a personal project which allows geotagging of images via GPX file in webassembly. It is serverless, so the browser does the work to geotag the images

# Development

```
project/
├─ assets/               # Static assets used by the app
├─ src/
│  ├─ main.rs            # The entrypoint for the app
│  ├─ components/
│  │  ├─ mod.rs          # Defines the components module
│  │  ├─ hero.rs         # The Hero component for the home page
│  │  ├─ sample_map.rs   # Map component for previewing geotagged locations
│  ├─ tagging/
│  │  ├─ mod.rs          # Defines the tagging module
│  │  ├─ gpx_reader.rs   # Parses GPX files to extract GPS track data
│  │  ├─ exif_tagger.rs  # Writes GPS coordinates into image EXIF metadata
├─ Cargo.toml            # Dependencies and feature flags
```

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve
```

Right now, the project only supports web, but can be made to be multiplatform pretty easily.

