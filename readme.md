![logo](logo_cnbg.png)
# `arpa` - a package for pulsar science in ARGOS
[![GitHub license](https://img.shields.io/badge/license-MIT-blue)](#license)
[![Crates.io Version](https://img.shields.io/crates/v/argos-arpa)](https://crates.io/crates/argos-arpa)
[![GitHub](https://badgen.net/badge/icon/github?icon=github&label)](https://github.com/SGullin/arpa)
[![docs](https://img.shields.io/docsrs/argos-arpa?logo=rust&style)](https://docs.rs/argos-arpa/latest/)

*This is still under development.*

Keeps track of TOAs and related things. This is mostly based off of TOASTER by Patrick Lazarus.

Most informations are split into data and metadata, e.g. `toa` and `toa_meta`. Parfiles and raw files are kept as files somewhere, and so only metadata is put in the DB.

## Usage
Add the library as such:
```
cargo add argos-arpa
```
Alternatively, fork either this repo or the [GUI](#gui).

To get started, you need to have a folder of `sql` files creating the tables you reference in the rust code, and a config `.toml` file. Both of their paths need to be given to `Archivist`'s constructor.

### New tables
If you fork this and want to add more tables, the [derive macro](https://github.com/SGullin/arpa-item-macro) might come in handy. The only necessities is that 
 1) the struct contains a field `id: i32`; and
 2) you add a new entry in the `Table` enum. 

In the future, support will be added for custom tables without forking.

## GUI
There is a GUI application developed for internal use, publicly available at https://github.com/SGullin/arpa-gui.

# License
`argos-arpa` is distributed under the terms of the [MIT License](LICENSE-MIT).

## Changelog
### 0.3.0
 - Prepared as library.
