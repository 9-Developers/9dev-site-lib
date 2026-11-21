# Third-party notices

The four flavor icons under `assets/images/icons/` are adapted from Microsoft
Fluent Emoji's High Contrast artwork: Sunflower (`latte.svg`), Potted Plant
(`frappe.svg`), Hibiscus (`macchiato.svg`), and Herb (`mocha.svg`).
[Upstream artwork at the source revision](https://github.com/microsoft/fluentui-emoji/tree/1ffb34c752ecf5d402f04cfb4b392c77f57c54bc/assets).
Fulla.fyi wrapped the paths in SVG symbols and replaced the fixed fill color
with `currentColor`. The SVGs are distributed here with those adaptations.

## Microsoft Fluent Emoji — MIT License

Copyright (c) Microsoft Corporation.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE

## Catppuccin palette — MIT License

`assets/catppuccin.css` contains the four [Catppuccin](https://github.com/catppuccin/catppuccin)
color palettes expressed in OKLCH.

Copyright (c) 2021 Catppuccin

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## HTMX 4.0.0 — Zero-Clause BSD

The `htmx` feature embeds an unmodified copy of the upstream
[`dist/htmx.min.js`](https://github.com/bigskysoftware/htmx/blob/v4.0.0/dist/htmx.min.js).
Its [license](https://github.com/bigskysoftware/htmx/blob/v4.0.0/LICENSE) reads:

Permission to use, copy, modify, and/or distribute this software for
any purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED “AS IS” AND THE AUTHOR DISCLAIMS ALL
WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES
OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE
FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY
DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN
AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT
OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

## Brand and interface icons

The non-theme symbols replace the corresponding 9dev.tech React icons. The
sources below are the upstream artwork used for **this crate**. Each SVG wraps
the artwork in a symbol named `icon` with inherited `currentColor`. The artwork
licenses do not grant trademark rights. Use brand marks to identify their
respective services and follow the owners' guidelines, including
[Discord's](https://discord.com/branding) and [GitHub's](https://brand.github.com/foundations/logo).

### Font Awesome Free 6.7.2 — CC BY 4.0

[`patreon.svg`](https://github.com/FortAwesome/Font-Awesome/blob/6.7.2/svgs/brands/patreon.svg)
is copyright 2024 Fonticons, Inc., and licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
This crate wraps the original path in a `currentColor` symbol and preserves the upstream viewBox.

### Bootstrap Icons v1.13.1 — MIT

[SVG sources](https://github.com/twbs/icons/tree/ce0e49dd063243118a115f17ad1fe1fe7576d552/icons)
and [license](https://github.com/twbs/icons/blob/ce0e49dd063243118a115f17ad1fe1fe7576d552/LICENSE).
The following files keep the upstream path geometry and viewBox:

- `bluesky.svg`, `discord.svg`, `display.svg`, `facebook.svg`, `github.svg`, `gitlab.svg`,
  `instagram.svg`, `linkedin.svg`, `mastodon.svg`, `rss.svg`, `signal.svg`, `spotify.svg`,
  `threads.svg`, `twitch.svg`, `youtube.svg`: same upstream filenames.
- `angle-down.svg`: `chevron-down.svg`; `website.svg`: `globe.svg`;
  `x.svg`: `twitter-x.svg`.

The MIT License (MIT)

Copyright (c) 2019-2024 The Bootstrap Authors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.

### Mono Icons — MIT

[`download.svg`](https://github.com/mono-company/mono-icons/blob/43fac75e3d5ff38a55588e8c94a02ed96003a6ac/svg/download.svg)
and [`email.svg`](https://github.com/mono-company/mono-icons/blob/43fac75e3d5ff38a55588e8c94a02ed96003a6ac/svg/email.svg)
retain the original paths, replacing their solid fill with inherited color.
[Upstream license and attribution](https://github.com/mono-company/mono-icons/blob/43fac75e3d5ff38a55588e8c94a02ed96003a6ac/README.md#license--attribution).

The MIT License (MIT)
Copyright 2020 Mono Company BV

This license pertains to the Mono Icon Font, the software, as described on http://icons.mono.company, NOT to any content created with the Mono Icon Font.

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

### AniList Submission Manual — MIT

`anilist.svg` is a monochrome trace of the
[official manual's logo.png](https://github.com/AniList/submission-manual/blob/c30e98afdb152869756458e94a3054041d107305/docs/.vuepress/public/logo.png)
under the [manual's license](https://github.com/AniList/submission-manual/blob/c30e98afdb152869756458e94a3054041d107305/LICENSE).
The background was removed and the two letters were traced together.

MIT License

Copyright (c) 2019 Josh Star

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## GitHub Octicons v12.0.0 — MIT

`git-branch.svg` wraps the [git-branch-24 paths](https://github.com/primer/octicons/blob/c65a99647735412c633053fed85b1c1f5eee3099/icons/git-branch-24.svg) in a `currentColor` symbol. This is a generic branch, not a hosting service logo.

MIT License

Copyright (c) 2021 GitHub Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
