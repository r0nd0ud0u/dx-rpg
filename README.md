# Dx-rpg

This a game based on lib-rpg framework for the back-end and dioxus for the front-end

- Lib-rpg repository: https://github.com/r0nd0ud0u/lib-rpg

## Contributing
### Dioxus
Install and update dioxus cli
```bash
cargo install --git https://github.com/DioxusLabs/dioxus dioxus-cli --locked
```

or 

`cargo binstall dioxus-cli@0.7.2 --force`

### Lib-rpg
Add the following lines to fetch lib-rpg:
    `Windows: %USERPROFILE%\.cargo\config.toml`
    `Unix: $HOME/.cargo/config.toml`
```
[net]
git-fetch-with-cli = true
```

To use rand functions from lib-rpg "get_random_nb" compilation on target32-wasm is necessary.
Add `getrandom = { version = "0.3", features = ["wasm_js"] }` to ./cargo.toml.

Following [instructions from rust docs.](https://docs.rs/getrandom/#webassembly-support), add in your  `.cargo/config.toml` the lines below:

```
# It's recommended to set the flag on a per-target basis:
[target.wasm32-unknown-unknown]
rustflags = ['--cfg', 'getrandom_backend="wasm_js"']
```


## Launch
Launch the Dioxus Fullstack app (do not forget to update dioxus-cli and cargo):

```bash
dx serve --platform web
```
- Open the browser to http://localhost:8080

## Deployment 

- Build ` dx bundle --platform web`

- On windows, run `dx-rpg\target\dx\dx-rpg\release\web\server.exe`

- Then access [http://localhost:8080](http://localhost:8080)

