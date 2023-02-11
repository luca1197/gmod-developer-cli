# ⌨️ gmod-developer-cli (`gcli`)

A CLI tool which assists you in creating Garry's Mod content.

⚠️ **This tool is in a very early barebone stage. Also, this is my first time actually using Rust, I (mostly) don't have a clue what I'm doing.** This project is primarily for me to get familiar with Rust, but perhaps someone else will find it useful. I created this tool because I was frustrated with constantly having to copy and paste files, then strip most of the content out of them just to create e.g. an entity.

## Installation

Download the latest release and move it into a directory which is inside your systems `PATH`.

### Detailed instructions
1. Download the [latest release](https://github.com/luca1197/gmod-developer-cli/releases/latest).
2. Create a directory in an appropriate place, e.g. `\Users\<username>\.path`.
3. Move the downloaded `gcli.exe` into the new directory.
4. Add the new directory to your `PATH`. For Windows:
   1. In Windows search, type `env`.
   2. Select "Edit environment variables".
   3. Click "Environment variables" in the bottom right.
   4. Select `PATH` / `Path` → "Edit".
   5. Click "New".
   6. Enter the path to the created directory, e.g. `C:\Users\<username>\.path`.
   7. Press "OK" in all opened windows.
5. You can now use `gcli` in new terminal sessions.

## Commands

### `addon`
`gcli addon init <target_directory>` » Initialises an addon by creating an `addon.json` file in the target directory with the specified values.

### `entity`
`gcli entity create <directory_name>` » Creates a barebone entity in the current addon directory. There are currently two entity templates to choose from - A basic physics entity and a NPC entity.