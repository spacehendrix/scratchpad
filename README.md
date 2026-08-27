# <img src="src-tauri/icons/128x128.png" alt="Scratchpad icon" width="32" /> Scratchpad

A terminal-style, keyboard-driven notes and checklist app for macOS — built for
thoughts that are meant to be **temporary**. Offload a checklist from your
head, draft something rough, and let it fade away on its own. Encrypted at
rest, local-only, no backend, no telemetry.

```
┌───────────────────────┐
│  s c r a t c h p a d  │
└───────────────────────┘
```

## The concept

Most note apps assume everything you write deserves to live forever. Scratchpad
assumes the opposite: notes are working memory, not an archive.

- **Capture fast.** One keystroke creates a note. Titles are optional (and
  usually absent — the first line stands in). Lines like `[ ] milk` become
  toggleable checkboxes.
- **Let go automatically.** A note you stop editing quietly leaves the main
  list after **30 days** (archived — still findable via search). After
  **3 years**, or when storage passes **5 GB**, the oldest expendable notes
  are permanently deleted.
- **Pin what matters.** A pinned note (`◆`) never archives and is never
  auto-deleted. Pinning is the explicit "keep this forever" signal; everything
  else sunsets.
- **Reading is not keeping.** Only *edits* (including checkbox toggles) reset
  a note's clock. Opening an archived note just reads it; editing it revives
  it into the main list.

## Security model

- Every document is encrypted with **XChaCha20-Poly1305** (authenticated
  encryption) before touching disk. The SQLite database stores only
  ciphertext — metadata included. Each blob is bound to its document id, so
  ciphertexts can't be swapped around.
- The random 256-bit master key lives in your **macOS login keychain** and is
  gated behind a **Touch ID / account-password** check on every unlock
  (`LocalAuthentication`, device-owner policy). Because the key is random —
  not derived from your password — changing your macOS password changes
  nothing.
- The key is held in zeroized memory only while the app is unlocked
  (`⌘L` locks instantly; quitting locks too) and never crosses into the UI
  layer.
- **If the keychain item is lost, the data is gone.** That's by design for an
  ephemeral-notes app: there is no recovery path, only an explicit
  "start fresh" flow that quarantines (renames, never deletes) the old
  database.
- The only plaintext on disk is `settings.json` (theme, font, dashboard
  preferences) — nothing sensitive.

Data lives in `~/Library/Application Support/com.spacehendrix.scratchpad/`.
Deleting the app does not delete your notes or key.

## How it works day to day

Unlock with Touch ID and you land on the **browse** list: pinned notes first,
then most-recently-edited, each with a relative age and a `[3/7]` checklist
badge. Above it, an ascii **dashboard** — a 14-day activity sparkline, a task
completion gauge, and a storage gauge (all configurable). Below, a status bar
with live stats and the key hints for wherever you are.

The **editor** is plain monospace text with a styled overlay: `#`/`##`
headings, `-`/`*` bullets, and checkboxes you can toggle with `⌘⏎` (caret
line) or by clicking the `[ ]` glyph. Autosave is debounced — there is no
save button. Empty new notes are simply never persisted.

**Search** (`/` or `⌘K`) is instant over titles/previews and streams through
full note bodies, decrypting one at a time in memory. `⇥` flips between the
active shelf and the **archive** — the only place archived notes appear.

### Keys

| Where  | Keys |
|--------|------|
| browse | `n` new · `⏎` open · `j`/`k` move · `p` pin · `d`→`y` delete · `/` search · `,` settings |
| editor | `esc` back · `⌘⏎` toggle checkbox |
| search | type to search · `⇥` archive scope · `↑`/`↓` · `⏎` open · `esc` back |
| settings | `j`/`k` move · `␣` select/toggle · `h`/`l` adjust size · `⏎` apply · `esc` cancel |
| anywhere | `⌘L` lock now |

### Settings

`,` opens settings: **21 themes** (tokyo night, catppuccin, gruvbox, nord,
dracula, and a true-black set including amber/green CRT phosphor looks),
**7 monospace fonts**, root **font size** (the whole UI scales), and the
**dashboard** panels/glyph-style/size. Everything previews live; `⏎` persists.

## Architecture

Strict two-layer split, so the UI can be swapped without touching the domain:

```
src-tauri/src/core/   Rust, zero Tauri imports — the actual app
  crypto.rs           XChaCha20-Poly1305 seal/open, zeroized keys
  keychain.rs         key storage + LocalAuthentication gate
  store.rs            encrypted SQLite (rusqlite, WAL, transactional)
  retention.rs        archive / age / space-eviction policy (injectable clock)
  search.rs           catalog scan + streaming decrypt-and-scan
  state.rs            Locked / Unlocked session, document operations
  settings.rs         plain settings.json (atomic writes)

src-tauri/src/        thin Tauri shell: commands.rs adapters + specta bindings
src/                  Svelte 5 frontend — purely presentational
```

Types cross the IPC boundary once, generated from Rust via
[tauri-specta](https://github.com/specta-rs/tauri-specta) into
`src/lib/bindings.ts`.

## Building

Requires Rust, Node 22+, and pnpm. macOS only.

```sh
pnpm install
pnpm tauri dev                      # development (hot reload)
pnpm tauri build --bundles app,dmg  # release: .app + installer dmg
cargo test        # (in src-tauri/) core tests: crypto, store, retention, search
pnpm test         # frontend tests: keyboard, checklist, formats, themes
```

The build is ad-hoc signed for local use — on first launch, right-click →
Open (or approve in System Settings → Privacy & Security).
