# Kyutile
A Neovim inspired terminal editor for Battle Dome map files, written in Rust.

## Installation
1. If you haven't already, install rustup and cargo as described [here](https://doc.rust-lang.org/cargo/getting-started/installation.html).
2. ```cargo install --git https://github.com/StellaHalf/kyutile.git```

## Usage

### Overview

Provided you have the .cargo/bin folder on the PATH you can run the editor with ```kyutile```, this
will create an empty 11x11 map. You can quit out of the editor anytime using `:q`.
You can also run ```kyutile <path>``` to open a file, or ```kyutile --version```
to view the version of the installed binary.

Inside the editor, you can open the command menu by pressing the `:` key, you can then type a command
from the command list below and execute it with the `return` key. Additionally, there are keyboard shortcuts for certain commands.

The most basic way to using kyutile is as follows:
1. Create an empty map using `:n <width> <height>`, or open a file using `:o path`.
2. Select a tile using `:t <tile>` (for example `:t stop`).
3. Move the cursor using either the arrow keys or the `hjkl` keys.
4. Set the tile at the cursor using the `d` key.
5. Save your work using `:w <path>` and quit using `:q`. Once you have saved or if you began with `:o`, just `:w` suffices.

### Command List
Each command has a name, some have an alias or arguments as well. Optional arguments will be marked with a question mark.
Commands can also depend on or change the following settings:
- **cursor**: shown as two red arrows
- **selection**: shown as blue backslashes 
- **current path**: the path last opened from or saved to
- **brush**: can be `add`, `subtract` or a tile. `add` and `subtract` let you add or remove from the selection respectively, and a tile lets you place tiles.
- **pen** mode: can be `Up` or `Down`, if `Down` then moving around will draw automatically.

| Name       | Alias | Arguments                           | Function                                                                                            |
| ---------- | ----- | ----------------------------------- | --------------------------------------------------------------------------------------------------- |
| open       | o     | \<path\>                            | Opens a file to edit, fails if there are unsaved changes.                                           | 
| open!      | o!    | \<path\>                            | Opens a file to edit and discards unsaved changes.                                                  |
| create     | n     | \<width\> \<height\>                | Creates a new empty map with the given dimensions.                                                  |
| write      | w     | \<path\>?                           | Saves the current map to the path, or to the **current path** if none is given.                     |
| quit       | q     |                                     | Exits the editor, fails if there are unsaved changes.                                               |
| quit!      | q!    |                                     | Exits the editor and discards unsaved changes.                                                      |
| write-quit | wq    | \<path\>?                           | Saves the current map to the path and then exits the editor.                                        |
| brush      | t     | `add`\|`subtract`\|\<tile\>         | Sets the **brush**.                                                                                 |
| goto       | g     | \<x\> \<y\>                         | Sets the cursor to the given position.                                                              |
| select     | s     | `all`|`none`|`invert`|\<tile\>      | Respectively **selects** everything, nothing, inverts the selection or all tiles of the given type. |
| box        | b     | \<x0\> \<y0\> \<x1\> \<y1\> `fill`? | Draws a rectangle at the given coordinates, fills it if `fill` is given.                            |
| ellipse    | e     | \<x0\> \<y0\> \<x1\> \<y1\> `fill`? | Draws an ellipse at the given coordinates, fills it if `fill` is given.                             |
| fuzzy      | f     | \<length\>?                         | Fills on tiles of the same type connected to the cursor, limited by a length if given.              |
 
The following commands exist for the sake of completeness, but are recommended to be accessed using keybinds instead and therefore don't have aliases.

| Name      | Arguments                                    | Function                                                                                            |
| --------- | -------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| dot       |                                              | Draws at the cursor position.                                                                       |

### Keybind List
