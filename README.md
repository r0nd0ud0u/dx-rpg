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

`cargo binstall dioxus-cli@0.7.3 --force`

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
#### Before login
<img width="1909" height="431" alt="image" src="https://github.com/user-attachments/assets/09e2e271-29fa-4d1a-a13c-4fb26255f2b4" />

#### After login
<img width="1904" height="346" alt="image" src="https://github.com/user-attachments/assets/bc050e92-1b10-4293-ac90-4dc8e706e622" />

#### Create server page
<img width="1899" height="253" alt="image" src="https://github.com/user-attachments/assets/073c230f-7344-48d0-b75f-75121b36f2d2" />

##### New game page

<img width="1911" height="268" alt="image" src="https://github.com/user-attachments/assets/4b7284e2-ee13-4ff8-9ac1-31742ed2cf1a" />

##### Load game page

<img width="1903" height="416" alt="image" src="https://github.com/user-attachments/assets/86e63864-2df6-4862-a95d-fe3eb41422e9" />

#### Join game page

<img width="1916" height="189" alt="image" src="https://github.com/user-attachments/assets/541fd928-f228-4c35-a7d4-a4ca86b3b5a2" />

