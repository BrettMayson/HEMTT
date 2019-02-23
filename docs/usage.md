# HEMTT Usage

<pre>
Usage:
    hemtt <a href="/HEMTT/#/usage?id=init">init</a>
    hemtt <a href="/HEMTT/#/usage?id=create">create</a>
    hemtt <a href="/HEMTT/#/usage?id=addon">addon</a> &lt;name&gt;
    hemtt <a href="/HEMTT/#/usage?id=build">build</a> [<a href="/HEMTT/#/usage?id=addons">&lt;addons&gt;</a>] [<a href="/HEMTT/#/usage?id=-release">--release</a>] [<a href="/HEMTT/#/usage?id=-force">-f</a>] [<a href="/HEMTT/#/usage?id=-nowarn">--nowarn</a>] [<a href="/HEMTT/#/usage?id=-opts">--opts</a>=&lt;addons&gt;] [<a href="/HEMTT/#/usage?id=-skip">--skip</a>=&lt;addons&gt;]
    hemtt <a href="/HEMTT/#/usage?id=clean">clean</a> [--force]
    hemtt <a href="/HEMTT/#/usage?id=run">run</a> &lt;utility&gt;
    hemtt <a href="/HEMTT/#/usage?id=update">update</a>
    hemtt (-h | --help)
    hemtt --version

Options:
    -f --force          Overwrite target files
       --nowarn         Suppress armake2 warnings
       --opts=&lt;addons&gt;  Comma seperated list of addtional compontents to build
       --skip=&lt;addons&gt;  Comma seperated list of addons to skip building
    -h --help           Show usage information and exit
       --version        Show version number and exit
</pre>
<hr/>

# init

Initialize a project file in the current directory. `init` is used when you have existing files or do not want to use the CBA structure.
<hr/>

# create

Create a new project using the CBA project structure. `create` should only be used inside an empty directory. The following structure will be generated.
<pre>
.
├── hemtt.json
└── <a href="https://github.com/synixebrett/HEMTT-Example/tree/master/addons">addons/</a>
    └── <a href="https://github.com/synixebrett/HEMTT-Example/tree/master/addons/main">main/</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/main/%24PBOPREFIX%24">$PBOPREFIX$</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/main/config.cpp">config.cpp</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/main/script_component.hpp">script_component.hpp</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/main/script_macros.hpp">script_macros.hpp</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/main/script_mod.hpp">script_mod.hpp</a>
        └── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/main/script_version.hpp">script_version.hpp</a>
</pre>
<hr/>

# addon

Create a new addon folder. Requires a name to be used for the addon.

```
./hemtt addon common
```
<pre>
.
└── <a href="https://github.com/synixebrett/HEMTT-Example/tree/master/addons">addons/</a>
    └── <a href="https://github.com/synixebrett/HEMTT-Example/tree/master/addons/common">common</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/common/%24PBOPREFIX%24">$PBOPREFIX$</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/common/CfgEventHandlers.hpp">CfgEventHandlers.hpp</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/common/XEH_PREP.hpp">XEH_PREP.hpp</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/common/XEH_postInit.sqf">XEH_postInit.sqf</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/common/XEH_preInit.sqf">XEH_preInit.sqf</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/common/XEH_preStart.sqf">XEH_preStart.sqf</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/common/config.cpp">config.cpp</a>
        ├── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/common/script_component.hpp">script_component.hpp</a>
        └── <a href="https://github.com/synixebrett/HEMTT-Example/tree/master/addons/common/functions">functions/</a>
            └── <a href="https://github.com/synixebrett/HEMTT-Example/blob/master/addons/common/functions/script_component.hpp">script_component.hpp</a>
</pre>
<hr>

# build
Build the project into PBO files. HEMTT will only build the files that have changed.

## addons
A comma seperated list of addon to build. HEMTT will build all addons in the `./addons` folder if no addons are specified. HEMTT will always build all addons when using `--release`.

**Build all**  
`hemtt build`

**Build a single addon**  
`hemtt build tracers`

**Build multiple addons**  
`hemtt build tracers,hearing`

## --nowarn
Hide warnings from the armake2 build process.

## --force
Remove existing built files before starting the next build.

## --release
Create and sign a release build of the project.

A `hemtt.json` file of 
```json
{
  "name": "Test Mod",
  "prefix": "TST",
  "author": "SynixeBrett",
  "files": [
    "mod.cpp"
  ]
}
```
would produce
<pre>
.
└── releases/
    └── 0.1.0.0/
        └── <a href="https://github.com/synixebrett/HEMTT-Example/tree/master/releases/0.1.0.0/%40TST">@TST/</a>
            ├── mod.cpp
            └── addons/
                ├── TST_common.pbo
                ├── TST_example.pbo
                └── TST_main.pbo
</pre>
This example is from the [HEMTT Example Project](https://github.com/synixebrett/HEMTT-Example)

## --opts
A comma seperated list of addtional addons to build. HEMTT will look for these in the `./optionals` folder. Using `--opts all` will build all addons in the `./optionals` folder.

`hemtt build --opts all`  
`hemtt build --opts tracers`  
`hemtt build --opts tracers,patrticles`

## --skip
A comma seperated list of additonal addons to skip building.

`hemtt build --skip hearing`  
`hemtt build --skip hearing,zeus`

<hr/>

# clean
Cleans all the files generated from previous builds.
<hr>

# run
Run a [Utility](/utilities.md).
<hr/>

# update

HEMTT will look for a more recent version online. If one is available you will be prompted to download the updated version.
