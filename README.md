# LTK Manager for macOS

**[Download the latest release (macOS, Apple Silicon)](https://github.com/aphelios623/LTK-mac/releases/latest)**

> **Unofficial macOS (Apple Silicon) build.** This is a community port of [LTK Manager](https://github.com/LeagueToolkit/ltk-manager) by the [League Toolkit](https://github.com/LeagueToolkit) team, based on [pull request #202](https://github.com/LeagueToolkit/ltk-manager/pull/202), which adds Apple Silicon macOS support. All credit for LTK Manager goes to the League Toolkit team and to the author of that pull request. **This project isn't affiliated with or endorsed by the LeagueToolkit team or Riot Games.**

LTK Manager is the frontier mod manager for League of Legends. Install custom skins, HUDs, maps, audio and much more. One of the main goals is to support mods so they work indefinitely without breaking between patches. It is the [League Toolkit](https://github.com/LeagueToolkit) team's successor to [cslol-manager](https://github.com/LeagueToolkit/cslol-manager).

If you make mods, the built-in Workshop is where you browse the game's files, edit your mod and pack it, without leaving the window.

## About this build

- For macOS on Apple Silicon (M-series) only.
- Unsigned, so macOS blocks it on first launch. Right-click the app, choose **Open**, then **Open** again. If macOS says the app is damaged, run `xattr -cr "/Applications/LTK Manager.app"` and try again.
- Asks for your administrator password each time the patcher starts and stops.
- Mods built for a different game version can crash League.

## Install

1. Download the `.dmg` from the [Release page](https://github.com/aphelios623/LTK-mac/releases/latest).
2. Drag **LTK Manager** into Applications and open it (see the first-launch note above).
3. Open Settings and set the League path to `/Applications/League of Legends.app/Contents/LoL` if auto-detect doesn't find it.

## Disclaimer

- Modifying game files is against Riot's terms, and use is at the user's own risk.
- This project isn't affiliated with or endorsed by the LeagueToolkit team or Riot Games.

## Credits and license

LTK Manager is developed by the [League Toolkit](https://github.com/LeagueToolkit) team: https://github.com/LeagueToolkit/ltk-manager. The macOS support comes from their pull request #202. The original license applies; see the `LICENSE` file.
