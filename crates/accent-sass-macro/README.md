# accent-sass-macro

The proc macro behind `accent_sass::include!`, which compiles Sass to CSS at
build time. It is an implementation detail of
[`accent-sass`](https://github.com/zoosky/accent-sass); depend on that crate
with the `macro` feature rather than on this one.

Renamed from `include_sass` when the project became `accent-sass`; the crate
of that name on crates.io belongs to upstream `grass`.
