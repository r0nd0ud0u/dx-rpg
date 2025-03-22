# Dx-rpg

This a game based on lib-rpg framework for the back-end and dioxus for the front-end

- Lib-rpg repository: https://github.com/r0nd0ud0u/lib-rpg

## Contributing
### Dioxus
Install and update dioxus cli
```bash
cargo install --git https://github.com/DioxusLabs/dioxus dioxus-cli --locked
```

### Lib-rpg
Add the following lines to fetch lib-rpg:
    `Windows: %USERPROFILE%\.cargo\config.toml`
    `Unix: $HOME/.cargo/config.toml`
```
[net]
git-fetch-with-cli = true
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

