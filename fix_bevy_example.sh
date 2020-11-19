grep bevy_webgl2::DefaultPlugins examples/ -r -L | xargs sed -i 's/DefaultPlugins/bevy_webgl2::DefaultPlugins/'
