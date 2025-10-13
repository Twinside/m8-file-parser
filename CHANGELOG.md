# Changelog

## 0.5.1

 - Fixed Version writing.
 - Adding method to create fresh tables/phrase from a version.

## 0.5

 - Analyzed/Fixed parsing of effect and mixer settings (with new elements)
 - Handling of firmware 6.2 (OTT and new added commands)

## 0.4 (hard fork)

 - Forking the repostiory from m8-files to m8-file-parser
 - FW 6.0 support

## 0.3.1

 - Fixing visibility of Eq types (was private in 0.3)
 - FmAlgo content is now publicly visible
 - Each instrument filter type can now be accessed through the filter_types method
 - Parameters and modulation destination constants is now public.

## 0.3

- v4 reading
- v4 overwriting, you can load, modify elements and rewrite the same song.
  * Does not work with song other than v4/v4.1
- Added EQ with plotting
- Mapped all FX instruction depending on the names
- Mapped enums to many instrument parameters with human readable information

## 0.2

- Add V3 support
- Fix instrument alignment issues
