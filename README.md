# BetterSteamAutoCracker

BetterSteamAutoCracker is a Tauri-based desktop app for Windows that automates Steamless DRM removal and Goldberg Steam Emulator application for offline Steam game use.

**Legal Notice**: For educational and personal use only (e.g., playing games offline so you "own" them). Bypassing DRM or emulating Steam services violates Steam's Terms of Service and laws.

## Features

- Removes Steam DRM with Steamless
- Applies Goldberg Steam Emulator for achievements, DLCs, and languages
- Auto-downloads dependencies (DLLs, fonts, sounds) from S3
- Backs up original files as `.svrn` and `Goldberg.zip`

## Building from Source

**Prerequisites**:

- Rust (via [rustup](https://rustup.rs/))
- [Node.js](https://nodejs.org/en) (v16+) and [Bun](https://bun.sh/)

**Steps**:

1. Clone the repo:

   - `git clone https://github.com/0xSovereign/BetterSteamAutoCracker.git`

2. Change the directory:

   - `cd BetterSteamAutoCracker`

3. Install dependencies:

   - `bun install`

4. Edit config:

   - Go to `src-tauri/src` and rename `config.rs.example` to `config.rs` and include the API key

5. Build:
   - `bun run tauri build`

And the executable will be in `src-tauri/target/release`

## Known Issues

- Some games with unique DLC setups may fail; report App IDs.
- File locks may require retries (up to 5 attempts).
- Denuvo-protected games need manual cracking first.

## Credits

- **Detanup01 & Mr.Goldberg**: Goldberg Steam Emulator
- **atom0s**: Steamless DRM removal
- **Sovereign**: BetterSteamAutoCracker developer

## License

MIT License. See [LICENSE](LICENSE).

## Disclaimer

Provided "as is" without warranty. Developers are not liable for damages or legal issues.

---

_Built with ❤️ by Sovereign using Tauri, Rust, and React with TailwindCSS._
