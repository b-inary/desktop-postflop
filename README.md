# Desktop Postflop

**Desktop Postflop** is a free, open-source GTO solver for Texas hold'em poker.

This is a port of [WASM Postflop] to a native desktop application using the [Tauri] framework.
Since WASM Postflop works on web browsers, it is more suitable for casual use.
However, if you want to use the solver for more serious purposes, please consider trying Desktop Postflop.

[WASM Postflop]: https://github.com/b-inary/wasm-postflop
[Tauri]: https://tauri.app/

**Related repositories**
- Solver engine: https://github.com/b-inary/postflop-solver

## Comparison to WASM Postflop

- **Shared features**:
  - Free to use and open-source
  - Same solver engine and user interface
- **Advantages**:
  - Faster computation
  - Able to use more than 4GB of memory
- **Disadvantages**:
  - Distributes only Windows builds
  - Needs to download the program and trust the execution

See the [WASM Postflop repository] for more detailed comparisons, including some commercial solvers.

[WASM Postflop repository]: https://github.com/b-inary/wasm-postflop#comparison

## Supported environments

- OS: Windows 10/11
- CPU: must support AVX instructions
  - Intel: Sandy Bridge and later
  - AMD: Zen (1st gen) and later

Though we only distribute Windows builds, we have confirmed that the app can be built and run on macOS.

## Download

You can download the installer (.msi) or the portable executable (.exe) from the [GitHub releases page].

[GitHub releases page]: https://github.com/b-inary/desktop-postflop/releases

- The installer version automatically installs dependent runtimes.
- To launch the portable version, the WebView2 runtime needs to be installed.
  - Please download and run the Evergreen Bootstrapper from [Microsoft's website].
  - Note: WebView2 is pre-installed in Windows 11

[Microsoft's website]: https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section

## Uninstall

To completely uninstall Desktop Postflop, please remove the following folder after the regular uninstallation:

```
C:\Users\<username>\AppData\Local\b-inary.desktop-postflop
```

## Build

Please see the [Tauri documentation].

[Tauri documentation]: https://tauri.app/v1/guides/getting-started/prerequisites

## License

Copyright (C) 2022 Wataru Inariba

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program.  If not, see <https://www.gnu.org/licenses/>.
