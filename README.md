# WebGL2 rendering backend for Bevy game engine

**bevy_webgl2** is external pluging for Bevy providing WebGL2 rendering backend on wasm target. To see it in action take a look on live [examples](https://mrk.sed.pl/bevy-showcase/#contributors)

## Building examples

### Prerequisites

```
cargo install cargo-make
```

to build and serve `sprite` example do:
```
cargo make example sprite --profile release
```

and open `http://127.0.0.1:4000`

## Using **bevy_webgl2** in your project

Take a look on [bevy_webgl2_app_template](https://github.com/mrk-its/bevy_webgl2_app_template) - it is a template of bevy application using cargo-make for building both native and WASM targets.

Latest released version of Bevy is 0.3.0 and it do not work with bevy_webgl2, so currently bevy_webgl2 uses latest known-to-work revision of Bevy from git. To use bevy_webgl2 in your project you have to add to `Cargo.toml` dependencies exactly the same version of Bevy that is used by bevy_webgl2 or patch it by adding `[patch]` section if you want to use newer version:

```
[dependencies]
bevy = {git = "https://github.com/bevyengine/bevy", branch="master", default-features=false, features=["bevy_winit", "render", "bevy_gltf", "png"]}
bevy_webgl2 = {git = "https://github.com/mrk-its/bevy_webgl2"}

[patch.'https://github.com/bevyengine/bevy']
bevy = {git = "https://github.com/bevyengine//", branch="master", default-features=false, features=["bevy_winit", "render", "bevy_gltf", "png"]}
```

(notice double slash at the end of patched bevy's git url - this is workaround for this Cargo's issue https://github.com/rust-lang/cargo/issues/5478)

To initialize plugin simply replace Bevy's `DefaultPlugins` with `bevy_webgl2::DefaultPlugins`:
```
    App::build()
        .add_plugins(bevy_webgl2::DefaultPlugins)
```
or add Bevy `DefaultPlugins` and `bevy_webgl2::WebGL2Plugin`
```
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_webgl2::WebGL2Plugin)
```
