# Cirula

Cirula (simple rust launcher) is an app launcher for wayland.
Currently, the only feature is launching apps from `.desktop` files.
Feel free to submit pull requests for any feature you like.

Fork of [Sirula](https://github.com/DorianRudolph/sirula) that replaces GTK with Iced.

## Why fork it?

I liked the simplicity of Sirula, but it's not been maintained for a while. I'm also more familiar with Iced than GTK, and it has fewer non-rust dependencies. This makes it easier to actively fix dependencies.

## Does it support everything Sirula supports

I have not thoroughly tested every option yet. The main goal was to have a working launcher to replace the one I used. :)

## Examples

Coming soon! This is still Work in progress! Though, it's mostly the same minus the CSS. Colours/transparency can be controlled with the `app_theme`/`custom_palette` settings in the `config.toml` file.

## Building

- Dependencies: [iced](https://github.com/iced-rs/iced), [iced_layershell](https://github.com/waycrate/exwlshelleventloop)
- Build: `cargo build --release`
  - Optionally, `strip` the binary to reduce size
- Alternatively, install with `cargo install --path .`

## Configuration

Use `config.toml` your `.config/sirula` directory.
See `sample-config` for documentation.

## Built-in themes

You can set the theme in the config.toml file (snake-case):

```toml
app_theme: tokyo_night_storm
```

### Available themes

- light
- dark
- dracula
- nord
- solarized_light
- solarized_dark
- gruvbox_light
- gruvbox_dark
- catppuccin_latte
- catppuccin_frappe
- catppuccin_macchiato
- catppuccin_mocha
- tokyo_night
- tokyo_night_storm
- tokyo_night_light
- kanagawa_wave
- kanagawa_dragon
- kanagawa_lotus
- moonfly
- nightfly
- oxocarbon
- ferra

## Custom 'theme'

You can also set a custom colour palette, including transparency.

```toml
[custom_palette]
background = "#14141f"
text = "#f2f2fa"
primary = "#8066e6"
success = "#4dcc99"
danger = "#e64d66"
warning = "#e6b34d"
```

## Differences from Sirula

|               | Cirula | Sirula                            |
| ------------- | ------ | --------------------------------- |
| UI framework  | Iced   | GTK                               |
| Theming       | TOML   | GTK CSS / TOML                    |
| NixOS support | Yes    | Partial (many icons don't render) |

### Known, unsupported toml config:
- `close_on_unfocus` : Currently it will close if you press `ESC`
- `lines` :  Not implemented
- `markup_default` : Not implemented
- `markup_higlight` : Not implemented
- `markup_extra` : Not implemented

## Roadmap

- Re-add `close_on_unfocus`
- Implement a feature for controlling the [Niri](https://github.com/YaLTeR/niri) desktop environment
