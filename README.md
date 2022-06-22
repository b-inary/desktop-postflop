# Desktop Postflop

**Desktop Postflop** is a free, open-source GTO solver for Texas hold'em poker.

This is a port of [WASM Postflop] to native desktop applications using the [Tauri] framework.
Since WASM Postflop works on web browsers, it is more suitable for casual use.
However, if you want to use the solver for more serious purposes, please consider installing Desktop Postflop.

[WASM Postflop]: https://github.com/b-inary/wasm-postflop
[Tauri]: https://tauri.app/

## Download

You can download the installer from the [GitHub releases page].
Please note that the current version is highly experimental.

[GitHub releases page]: https://github.com/b-inary/desktop-postflop/releases

### Supported environments

- OS: Windows 7+, macOS 10.15+, Linux
- CPU: must support AVX instructions
  - Intel: Sandy Bridge and later
  - AMD: Zen (1st gen) and later
  - **Mac with M1/M2 chip is not supported**

## Build

Please see the [Tauri documentation].

[Tauri documentation]: https://tauri.app/v1/guides/getting-started/prerequisites

## License

Copyright (C) 2022 Wataru Inariba

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program.  If not, see <https://www.gnu.org/licenses/>.
