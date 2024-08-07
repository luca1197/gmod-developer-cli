# ⌨️ gmod-developer-cli (`gcli`)

A CLI tool which assists you in creating Garry's Mod content.

*Warning - Some commands of this tool are a "work in progress" and may not function fully as expected yet.*

## Installation

Download the latest release and move it into a directory which is inside your systems `PATH`.

### Detailed instructions (Windows)
1. Download the [latest release](https://github.com/luca1197/gmod-developer-cli/releases/latest) and unzip it.
2. Create a directory in an appropriate place, e.g. `\Users\<YOUR-USERNAME>\.path`.
3. Move the downloaded `gcli.exe` into the new directory.
4. Add the new directory to your `PATH`:
   1. In Windows search, type `env`.
   2. Select "Edit environment variables".
   3. Click "Environment variables" in the bottom right.
   4. In the upper list, select the variable named `PATH` / `Path`, then click "Edit".
   5. Click "New".
   6. Enter the path to the created directory, e.g. `C:\Users\<YOUR-USERNAME>\.path`.
   7. Press "OK" in all opened windows.
5. You can now use `gcli` in new terminal sessions.

## Commands

### `addon`
#### `gcli addon init <target_directory>`
Initialises an addon by creating an `addon.json` file in the target directory with the specified values.

### `entity`
#### `gcli entity create <directory_name>`
Creates a barebone entity in the current addon directory. There are currently two entity templates to choose from - A basic physics entity and a NPC entity.

### `vmf`
#### `gcli vmf collect-content <vmf_path>`
Collects the content a vmf (map) uses, looks for it in the provided source paths and copies it to the specified output directory.

This is very useful when using content from many different sources, since this will allow you to just use everything freely without having to worry about copying content manually to avoid missing models / materials.

Currently, this command only supports materials and models (no sounds). The command will parse materials and models to look for referenced materials and textures. [Patch materials](https://developer.valvesoftware.com/wiki/Patch) are supported.

This command will look at the game files to check if any content missing in the provided source directories is already part of the game. This will use the game's `gameinfo.txt`, so make sure that you did not mount any additional custom content in there since the command will assume that it is part of the game, thus not including in the output!

**Options:**
* `-s <source_path>` - Path to a directory which contains content the map potentially uses. This option can be used multiple times.
* `-o <output_path>` - Path to a directory where all of the content the map uses will be copied to.

Keep in mind that it is not rare to encounter many models that are missing materials. For example this may be caused by skin slots that have no material present, which is the fault of the model creator. If you encounter such warnings, just load the map in-game and check if anything is missing manually. In addition to that, some models do not have a physics model (`.phy`) which will cause warnings that you should fix or ignore on case-by-case basis (again, just test it in-game).

For model files, this command currently only copies the `.dx90.vtx`, `.mdl`, `.phy` and `.vvd` files since those are the only required files for a modern GMod install which reduces the final content file size.

## Building

Requires "C++ MFC for latest v143 build Tools (x86 & x64)", which can be installed using the Visual Studio Installer.

(TODO: Add more instructions)

## ❤️ Credits
- [lasa01/plumber_core](https://github.com/lasa01/plumber_core) (Forked to [luca1197/fork-plumber_core](https://github.com/luca1197/fork-plumber_core)) - Without this library the `vmf collect-content` command would be impossible due to how much work it would be to implement all of the parsing functionality this library offers. A real gem.
