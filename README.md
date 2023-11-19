# Plotters GTK4

[![github](https://img.shields.io/badge/github-seadve/plotters-gtk4)](https://github.com/SeaDve/plotters-gtk4)
[![crates.io](https://img.shields.io/crates/v/plotters-gtk4)](https://crates.io/crates/plotters-gtk4)
[![docs](https://docs.rs/plotters-gtk4/badge.svg)](https://docs.rs/plotters-gtk4/)
[![CI](https://github.com/SeaDve/plotters-gtk4/actions/workflows/ci.yml/badge.svg)](https://github.com/SeaDve/plotters-gtk4/actions/workflows/ci.yml)

Plotters GTK4 Backend

This is a third-party backend that allows plotters to operate with GTK4 drawing APIs. For more details, please check the following links:

- For a high-level intro of Plotters, see: [Plotters on crates.io](https://crates.io/crates/plotters)
- Check the main repo at [Plotters repo](https://github.com/38/plotters.git)
- For detailed documentation about this crate, check [plotters-backend on docs.rs](https://docs.rs/plotters-backend/)
- You can also visit Plotters [Homepage](https://plotters-rs.github.io)

## Examples

This crate provides two backend flavors:

### Snapshot Backend

This backend is similar to the `CairoBackend` from the [`plotters-cairo`](https://github.com/plotters-rs/plotters-cairo) crate. This is suitable if you are directly drawing to a `GtkSnapshot` or implementing your own widget or paintable.

### Paintable Backend

This is preferred if you simply want to display a plot using `GtkPicture` or any other APIs that accept a `GdkPaintable`.

For a real-world example, [Spicy](https://github.com/SeaDve/spicy), a GTK4 frontend for Ngspice circuit simulator, uses this backend to plot simulation results.

## License

Copyright 2023 Dave Patrick Caberto

This software is subject to the terms of the MIT License. If a copy of the MIT License was not distributed with this file, You can obtain one at [this site](https://opensource.org/license/mit/).
