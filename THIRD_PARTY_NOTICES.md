# Third-party notices

New FrameLab code is MIT licensed. Dependencies retain their original licenses.

- **PresentMon 2.6.0**, GameTechDev/Intel: MIT. The official x64 console executable is downloaded separately by the build script, pinned by size/SHA-256 in `tools/presentmon.lock.json`, and included in Windows packages alongside the complete upstream `LICENSE.txt`.
- **Tauri** and official plugins: MIT/Apache-2.0. **Rust** and Windows bindings: MIT/Apache-2.0. Dependency versions/checksums are in Cargo.lock.
- **React**, **Radix primitives**, **Lucide**, **Vite**, **Vitest**, **Playwright** and other JavaScript dependencies retain their package notices; versions/integrity are in package-lock.json. Playwright/axe are validation tools, not runtime game dependencies.
- **SQLite** is public domain; rusqlite retains its own notices. No model weights, commercial bot source, game executable, game assets or human recordings are bundled.

Synthetic CSV/config examples and the repository-native N icon are authored for this project. They are not captured CS2 benchmark data or copies of private user configurations.
