# Changelog

## [0.0.1-beta.8](https://github.com/livondevs/livon/compare/0.0.1-beta.7...0.0.1-beta.8) (2025-4-11)

### Features
- **Added For block functionality** [#73] @s19514tt
- **Updated attribute handling in $$livonInitComponent to support dynamic updates and boolean attributes** [#76] @s19514tt
- **Added lifecycle functions afterMount and afterUnmount** [#79] @s19514tt
- **Implemented dynamic binding for class objects** [#80] @s19514tt
- **Supported else-if blocks for if statements** [#81] @s19514tt
- **Introduced Reactive Function feature** [#82] @s19514tt
- **Added reactivity support for object changes** [#87] @s19514tt

### Fixes
- **Fixed issue where non-self-closing HTML elements were incorrectly self-closed** [#74] @s19514tt
- **Fixed bug where templates ending with a newline were not parsed correctly** [#75] @s19514tt
- **Fixed renderForBlock to accept items as a parameter and improved update logic** [#78] @s19514tt
- **Fixed issue with HTML element id removal by updating the bitwise operation in gen_binary_map_from_bool** [#83] @s19514tt
- **Fixed bit number overflow by updating implementation to use an array for combining bit numbers** [#84] @s19514tt
- **Added sort_order to AddStringToPosition and implemented sorting in analyze_js** [#85] @s19514tt
- **Fixed issues regarding sort_order and reactive function parser** [#86] @s19514tt
- **Fixed event binding statement to use 'e' instead of 'event'** [#88] @s19514tt
- **Enhanced version check workflow by improving output messages and streamlining the process** [#89] @s19514tt


## [0.0.1-beta.7](https://github.com/livondevs/livon/compare/0.0.1-beta.6...0.0.1-beta.7) (2025-3-8)

### Features
- Fixed the issue where multiple conditions could not be nested in an IF block. #54
- Added a formatter. #55
- Added TypeScript support. #58

### Refactor
- Simplified the code for component declarations. #60

### DevOps
- Automatically categorize PR templates. #62

## [0.0.1-beta.6](https://github.com/livondevs/livon/compare/0.0.1-beta.5...0.0.1-beta.6) (2024-7-28)

### Other Changes

- Changed the name of the project from `Blve` to `Lunas`. #46

## [0.0.1-beta.5](https://github.com/livondevs/livon/compare/0.0.1-beta.4...0.0.1-beta.5) (2024-7-28)

### Features

- Added feature to pass variables to child components. #38
- Added auto-routing feature. #43

### Bug Fixes
- Fixed the issue where top-level element attribute binding is not working. #35
- Fixed the issue of not deleting variable dependencies of component when unmounting. #44

### DevOps
- Added compiler server for development. #40
- Added automatic labels for issues. #41

## [0.0.1-beta.4](https://github.com/livondevs/livon/compare/0.0.1-beta.3...0.0.1-beta.4) (2024-6-7)

### Features

- Added feature to import external packages in the component file. #23
- Added feature to create custom components. #25 #29
- Added license file. #31

### Bug Fixes
- Fixed the issue where child if blocks are not rendered. #26
- Fix the issue where event listeners and text bindings are not rendered under if block. #28
- Fix the order of text node when rendered with if and custom block. #30

### DevOps
- Added git-pr-release action. #19 #20 #21

## [0.0.1-beta.3](https://github.com/livondevs/livon/compare/0.0.1-beta.2...0.0.1-beta.3) (2024-6-7)

### Features
- Added two-way data binding support.
- Added support for `if` block.

## [0.0.1-beta.2](https://github.com/livondevs/livon/compare/0.0.1-beta.1...0.0.1-beta.2) (2024-6-7)

### Features
- Attribute binding support

## [0.0.1-beta.1](https://github.com/livondevs/livon/tree/0.0.1-beta.1) (2024-6-7)

### Features
- Initial release with basic features
  - Support for text binding
  - Event binding support
