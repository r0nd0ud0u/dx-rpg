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
`cargo install cargo-binstall`

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

### Bundle
- Build ` dx bundle --platform web`

- On windows, run `dx-rpg\target\dx\dx-rpg\release\web\server.exe`
- Then access [http://localhost:8080](http://localhost:8080)

### Docker
Use scripts in `scripts` dir to build and run the app.
Use docker desktop under Windows.

## Introduction
### Home page
<img width="1917" height="415" alt="image" src="https://github.com/user-attachments/assets/4c9bb8bc-83ec-4749-83ed-1626f8147aa2" />

#### Create server page
<img width="1915" height="323" alt="image" src="https://github.com/user-attachments/assets/6e7bd0a3-7518-4383-8e89-e99acefe97c3" />

##### New game page

- lobby page
<img width="1918" height="271" alt="image" src="https://github.com/user-attachments/assets/ee1a7ae4-ead9-49f4-a156-b96fa569f7e5" />

Then

<img width="1872" height="913" alt="image" src="https://github.com/user-attachments/assets/e61b029b-0e72-4461-a068-1133074077d5" />

##### Load game page

<img width="1913" height="492" alt="image" src="https://github.com/user-attachments/assets/1540f9a5-37d0-43ed-9204-41a5ae1bda99" />


#### Join game page

<img width="1918" height="272" alt="image" src="https://github.com/user-attachments/assets/c487c7a2-6890-45be-9ae1-e4a9ced7e87c" />
