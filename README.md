# HEMTT

[![codecov](https://codecov.io/gh/BrettMayson/HEMTT/branch/experiment/graph/badge.svg?token=A6AG4OK9SH)](https://codecov.io/gh/BrettMayson/HEMTT)

Build System for Arma 3

## What is this branch?

This is a full rewrite of HEMTT with it's own config preprocessor. It also includes `pbo-rs`, as  that project will be deprecated when this is released.

The new HEMTT is modular, allowing certain pieces of it to be included in other projects. The modularity can also decrease compile times when working locally, as only the edited modules need to be recompiled.

## Why a rewrite?

The biggest issues in my opinion with HEMTT were the lack of error reporting and logging. Some of that had to do with the use of armake2 since that project wasn't intended to be used in this way. Building more modular, library-first components should allow HEMTT to be more easily provide information to the user about what is happening, and allow developers to easily expand it or incorporate it in their own projects.

Also it has tests now so that's alright.

## Roadmap

### 0.9

* [ ] hemtt
  * [ ] Building
  * [ ] Releasing
  * [ ] Scripts
* [ ] hemtt-app
  * [ ] Project Management
    * Templates
      * [x] CBA
      * [ ] ACE
      * [ ] Vanilla
    * [x] New Addon
    * [x] New Function
    * [x] Version Bumping
* [ ] hemtt-arma-config
  * [x] Preprocessor
    * [x] Defines
    * [x] If Else
  * [ ] Parser
    * [ ] All the fun usual stuff
    * [ ] Enums
* [x] hemtt-pbo
  * [x] Reading PBOs
  * [x] Writing PBOs
* [x] hemtt-handlebars

### Post 1.0

* [ ] hemtt-arma-config
  * [ ] Preprocessor
    * [ ] __EVAL
  * [ ] Linter
* [ ] hemtt-arma-sqf
  * [ ] Linter
  * [ ] Static Analysis
  * [ ] VM
