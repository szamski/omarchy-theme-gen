
<img src="https://github.com/refact0r/system24/raw/main/assets/preview.png">

# Omarcord

A Discord theme that automatically syncs colors with your [Omarchy](https://github.com/omarchy/omarchy) terminal theme system. Forked from [system24](https://github.com/refact0r/system24).

<img src="https://github.com/refact0r/system24/raw/main/assets/screenshot.png">

## What's Different from system24?

- **Automatic Color Sync**: Colors are automatically generated from your Omarchy theme
- **Dynamic Theme Switching**: Change your terminal theme, Discord follows instantly
- **Omarchy Integration**: Works seamlessly with `omarchy-theme-gen`

## Prerequisites

1. [Omarchy Theme System](https://github.com/omarchy/omarchy) installed and configured
2. [omarchy-theme-gen](https://github.com/szamski/omarchy-theme-gen) installed and running
3. Vencord or BetterDiscord

## Quick Start

### 1. Install Omarchy Theme Gen

```bash
git clone https://github.com/szamski/omarchy-theme-gen.git
cd omarchy-theme-gen
./install.sh
omarchy-theme-gen init-config
```

### 2. Edit the Theme File

Update the path in `theme/omarcord.theme.css` line 98 to match your username:

```css
@import url('file:///home/YOUR_USERNAME/.config/omarchy/current/theme/omarcord-colors.css');
```

### 3. Install the Theme in Discord

**Option A: Using install script (recommended)**
```bash
git clone https://github.com/szamski/Omarcord.git
cd Omarcord
./install-omarcord.sh
```

**Option B: Manual installation**
```bash
# Download the theme
wget https://raw.githubusercontent.com/szamski/Omarcord/main/theme/omarcord.theme.css

# Edit line 98 to match your username
# Change: file:///home/szamski/...
# To: file:///home/YOUR_USERNAME/...

# Copy to Vencord themes directory
cp omarcord.theme.css ~/.config/Vencord/themes/

# Or for BetterDiscord
cp omarcord.theme.css ~/.config/BetterDiscord/themes/
```

### 4. Start the Generator

```bash
omarchy-theme-gen once  # Generate once
# or
omarchy-theme-gen watch  # Auto-update on theme changes
```

### 5. Enable in Discord

Enable "Omarcord" in Discord theme settings and reload (Ctrl+R).

**For detailed setup instructions, see [OMARCHY_INTEGRATION.md](OMARCHY_INTEGRATION.md).**

---

# Original system24 Documentation

Below is the original system24 documentation. Omarcord maintains full compatibility with all system24 features.

## discord server

need help with system24? want to get notified about updates? have feedback? join <https://discord.gg/nz87hXyvcy>

## install (system24 standalone)

### vencord/betterdiscord (or any client that supports theme files)

1. download the theme file, [`system24.theme.css`](https://github.com/refact0r/system24/blob/main/theme/system24.theme.css). (there should be a download button at the top right of the page)
2. drag the file into your theme folder. (there should be a button to open the theme folder in theme settings)
3. (optional) customize the theme by editing the options in `system24.theme.css`.

### install through link

add `https://refact0r.github.io/system24/build/system24.css` to your theme import links. you will need to copy the theme variables to your quickcss in order to customize the theme.

## flavors

customized variants of the theme.

- [catppuccin mocha](https://github.com/refact0r/system24/blob/main/theme/flavors/system24-catppuccin-mocha.theme.css)
- [catppuccin macchiato](https://github.com/refact0r/system24/blob/main/theme/flavors/system24-catppuccin-macchiato.theme.css)
- [everforest](https://github.com/refact0r/system24/blob/main/theme/flavors/system24-everforest.theme.css)
- [rosé pine](https://github.com/refact0r/system24/blob/main/theme/flavors/system24-rose-pine.theme.css)
- [rose pine moon](https://github.com/refact0r/system24/blob/main/theme/flavors/system24-rose-pine-moon.theme.css)
- [tokyo night](https://github.com/refact0r/system24/blob/main/theme/flavors/system24-tokyo-night.theme.css)
- [nord](https://github.com/refact0r/system24/blob/main/theme/flavors/system24-nord.theme.css)
- [vencord](https://github.com/refact0r/system24/blob/main/theme/flavors/system24-vencord.theme.css)

## contributing

this theme depends on [midnight](https://github.com/refact0r/midnight-discord) for its core styles. if you're looking to contribute, please consider which theme you actually want to work on. feel free to open an issue and ask if you're unsure.

this theme uses a dev script to check for changes in the source css files and combine them into a build file. to run locally:

1. clone the repository.
2. run `npm i`.
3. create a `.env` file in the project root with the paths of any local theme files you want to update (comma separated)

```
DEV_OUTPUT_PATH=C:\Users\USERNAME\AppData\Roaming\Vencord\themes\system24-dev.theme.css
```

4. run `npm run dev`.
5. make changes to any file in `/src` or the main theme file. the local theme files you listed will automatically be updated, along with the build file in `/build`.
6. make a pull request with your changes!

## credits

[spicetify text theme](https://github.com/spicetify/spicetify-themes/tree/master/text) for primary design inspiration.

thanks to all the [contributors](https://github.com/refact0r/system24/graphs/contributors)!
