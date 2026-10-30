# Third-Party Notices

`nexrad-data` is licensed under the MIT license (see the repository's `LICENSE` file).
It also contains the following third-party code.

## recast-radar-bzip2 (vendored in `src/recast_bzip2/`)

The decoder half of the `recast-radar-bzip2` crate, version 0.1.3, from
<https://github.com/recastsystems/recast-radar-tools> (`crates/recast-radar-bzip2`) at git
revision `c206a2495c36341caa2a62ff7be3025320dbb028`, is vendored in
`src/recast_bzip2/` and compiled when the `recast-bzip2` feature is enabled (or when the
`bzip2` feature is disabled). The header of `src/recast_bzip2/mod.rs` lists the local
modifications.

The upstream crate is dual-licensed under `MIT OR Apache-2.0`. It is used here under the
MIT license, reproduced below.

```text
MIT License

Copyright (c) 2026 Fahrenheit Research

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
```

### bzip2/libbzip2 randomisation table

`src/recast_bzip2/rand.rs` contains the 512-entry block randomisation table from
bzip2/libbzip2 1.0.8 (`randtable.c`), Copyright (C) 1996-2019 Julian R Seward, used under
its BSD-style license. The required notice is retained at the top of that file; only the
table values are reused.
