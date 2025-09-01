![logo](logo_cnbg.png)
# `arpa` -- Pulsar data archive

Keeps track of TOAs and related things. This is mostly based off of TOASTER by Patrick Lazarus.

Most informations are split into data and metadata, e.g. `toa` and `toa_meta`. Parfiles and raw files are kept as files somewhere, and so only metadata is put in the DB.

## toast 
The `toast` program hosts the main pipeline functionality. It can take either a raw file fresh from observations, or a previously uploaded one, and will in the first case archive it. Then, it will perform the analysis and store the results in the database.

## Changelog
### 0.2.0
 * `toast` seems to have a fully functional core pipeline
 * Removed reliance on `CacheTable`
 * Added and edited misc. convenience functions

### 0.1.0
 * Now keeps track of raw files, and computes a checksum
 * Added ra & dec to `PulsarId`
 * Added `toast` binary with `Chef` to cook raw files
 * Added `par` to add or fit ephemerides
 * Added `psr` to manage pulsar metadata
