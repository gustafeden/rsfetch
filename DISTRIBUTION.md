# Distribution & Visibility

How to get rsfetch in front of people.

## Demo GIFs

Record with [VHS](https://github.com/charmbracelet/vhs). Tapes are in `demo/`.

```sh
vhs demo/fetch.tape        # standard fetch output
vhs demo/boot.tape          # boot mode (procedural starfield)
vhs demo/boot-image.tape    # boot mode with background image
```

Output goes to `demo/` as `.gif` files. Embed in README.

## Package Managers

### Homebrew (macOS)

- [x] Create `github.com/gustafeden/homebrew-tap` repo
- [x] Add formula that downloads release binary from GitHub
- [x] Users install with: `brew tap gustafeden/tap && brew install rsfetch`
- [ ] Once adoption grows, submit to homebrew-core for `brew install rsfetch`

### AUR (Arch Linux)

- [x] Create `PKGBUILD` for AUR
- [x] Submit as `rsfetch-gustafeden-bin` (binary package)
- [x] Users install with: `yay -S rsfetch-gustafeden-bin`
- [x] Deletion request submitted for abandoned `rsfetch-bin` package (awaiting approval)
- [ ] Once approved: migrate to `rsfetch-bin` for cleaner name

### crates.io

- [ ] `cargo publish` — lets Rust users `cargo install rsfetch`
- [ ] Already have metadata in Cargo.toml (license, repo, keywords, categories)
- [ ] Note: blaeck is a git dependency — may need to publish blaeck first or vendor it

## GitHub Discoverability

- [x] Add repo topics: `neofetch`, `fastfetch`, `system-info`, `fetch`, `terminal`, `rust`, `cli`
- [ ] Add GIFs to README (the boot mode is the differentiator)
- [x] Submit PR to [awesome-fetch](https://github.com/beucismis/awesome-fetch) - [PR #188](https://github.com/beucismis/awesome-fetch/pull/188)

## Community Posts

Once README has GIFs:

- [ ] r/unixporn — boot mode screenshots/GIFs
- [ ] r/rust — new Rust CLI tool
- [ ] r/commandline — fetch tool comparison
