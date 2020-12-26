# WebGL2 rendering backend for Bevy game engine

**bevy_webgl2** is external plugin for Bevy providing WebGL2 rendering backend on wasm target. To see it in action take a look on live [examples](https://mrk.sed.pl/bevy-showcase/#contributors)

## Building examples

### Prerequisites

```
cargo install cargo-make
```
```
rustup target add wasm32-unknown-unknown
```

to build and serve `sprite` example do:
```
cargo make example sprite --profile release
```

and open `http://127.0.0.1:4000`

## Using **bevy_webgl2** in your project

Take a look on [bevy_webgl2_app_template](https://github.com/mrk-its/bevy_webgl2_app_template) - it is a template of bevy application using cargo-make for building both native and WASM targets.

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
