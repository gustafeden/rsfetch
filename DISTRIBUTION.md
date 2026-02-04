# Distribution & Visibility

How to get blaeckfetch in front of people.

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
- [x] Users install with: `brew tap gustafeden/tap && brew install blaeckfetch`
- [ ] Once adoption grows, submit to homebrew-core for `brew install blaeckfetch`

### AUR (Arch Linux)

- [x] Create `PKGBUILD` for AUR
- [x] Submit as `blaeckfetch-bin` (binary package)
- [x] Users install with: `yay -S blaeckfetch-bin`

### crates.io

- [x] Already have metadata in Cargo.toml (license, repo, keywords, categories)
- [x] Switched to crates.io version of blaeck (was git dependency)
- [ ] Publish as `blaeckfetch` on crates.io — lets Rust users `cargo install blaeckfetch`

## GitHub Discoverability

- [x] Add repo topics: `neofetch`, `fastfetch`, `system-info`, `fetch`, `terminal`, `rust`, `cli`
- [x] Add GIFs to README (the boot mode is the differentiator)
- [x] Submit PR to [awesome-fetch](https://github.com/beucismis/awesome-fetch) as blaeckfetch - [PR #188](https://github.com/beucismis/awesome-fetch/pull/188)

## Community Posts

**Ready to post** (prepared in `POSTS.md`):

- [ ] Show HN — "blaeckfetch – 57x faster than neofetch, with retro boot mode"
- [ ] r/unixporn — boot mode screenshots/GIFs
- [ ] r/rust — technical post about performance + unique features
- [ ] X/Twitter — short demo with GIF

**Future posts:**

- [ ] r/commandline — fetch tool comparison
- [ ] Dev.to — "Building a System Fetch Tool 57x Faster Than Neofetch"
- [ ] Hashnode — "How I Made a Terminal Tool 57x Faster by Switching to Rust"
