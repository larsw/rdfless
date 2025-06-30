# rdfless pager keybindings

This documentation is copied from the `minus` crate [source code](https://github.com/AMythicDev/minus/blob/main/src/lib.rs).

## Default keybindings

Here is the list of default key/mouse actions handled by `minus`.

**A `[n] key` means that you can precede the key by an integer**.

| Action              | Description                                                                  |
|---------------------|------------------------------------------------------------------------------|
| Ctrl+C/q            | Quit the pager                                                               |
| \[n\] Arrow Up/k    | Scroll up by n number of line(s). If n is omitted, scroll up by 1 line       |
| \[n\] Arrow Down/j  | Scroll down by n number of line(s). If n is omitted, scroll down by 1 line   |
| Ctrl+h              | Turn off line wrapping and allow horizontal scrolling                        |
| \[n\] Arrow left/h  | Scroll left by n number of line(s). If n is omitted, scroll up by 1 line     |
| \[n\] Arrow right/l | Scroll right by n number of line(s). If n is omitted, scroll down by 1 line  |
| Page Up             | Scroll up by entire page                                                     |
| Page Down           | Scroll down by entire page                                                   |
| \[n\] Enter         | Scroll down by n number of line(s).                                          |
| Space               | Scroll down by one page                                                      |
| Ctrl+U/u            | Scroll up by half a screen                                                   |
| Ctrl+D/d            | Scroll down by half a screen                                                 |
| g                   | Go to the very top of the output                                             |
| \[n\] G             | Go to the very bottom of the output. If n is present, goes to that line      |
| Mouse scroll Up     | Scroll up by 5 lines                                                         |
| Mouse scroll Down   | Scroll down by 5 lines                                                       |
| Ctrl+L              | Toggle line numbers if not forced enabled/disabled                           |
| Ctrl+f              | Toggle [follow-mode]                                                         |
| /                   | Start forward search                                                         |
| ?                   | Start backward search                                                        |
| Esc                 | Cancel search input                                                          |
| n                   | Go to the next search match                                                  |
| p                   | Go to the next previous match                                                |

## Key Bindings Available at Search Prompt

| Key Bindings      | Description                                         |
|-------------------|-----------------------------------------------------|
| Esc               | Cancel the search                                   |
| Enter             | Confirm the search query                            |
| Backspace         | Remove the character before the cursor              |
| Delete            | Remove the character under the cursor               |
| Arrow Left        | Move cursor towards left                            |
| Arrow right       | Move cursor towards right                           |
| Ctrl+Arrow left   | Move cursor towards left word by word               |
| Ctrl+Arrow right  | Move cursor towards right word by word              |
| Home              | Move cursor at the beginning pf search query        |
| End               | Move cursor at the end pf search query              |
