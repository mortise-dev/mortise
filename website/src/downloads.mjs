// The installers of the latest release, and the short links of the site that point at them.
// The release workflow (.github/workflows/tag-release.yml) uploads each file under a name with no
// version, so GitHub's /releases/latest/download/<name> always serves the newest one.
// astro.config.mjs turns `slug` into the short link /mortise/download/<slug>, to share.
// Downloads.astro links to `url` itself.
const latest = 'https://github.com/mortise-dev/mortise/releases/latest/download';

export const downloads = [
  { slug: 'macos', label: 'macOS', detail: 'Apple silicon, .dmg', file: 'Mortise-macos-arm64.dmg' },
  { slug: 'windows', label: 'Windows', detail: 'x64, installer', file: 'Mortise-windows-x64-setup.exe' },
  { slug: 'linux', label: 'Linux', detail: 'x86_64, AppImage', file: 'Mortise-linux-x86_64.AppImage' },
  { slug: 'deb', label: 'Debian and Ubuntu', detail: 'amd64, .deb', file: 'Mortise-linux-amd64.deb' },
].map((d) => ({ ...d, url: `${latest}/${d.file}` }));
