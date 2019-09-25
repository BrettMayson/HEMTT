# Getting Started

## Creating a New Mod

Setting up a new mod is quick and simple. HEMTT will create a template that uses [CBA](https://github.com/cbateam/cba_a3). Your mod will therefore require the [CBA Mod](https://steamcommunity.com/workshop/filedetails/?id=450814997). The CBA Mod is small, light and provides lots of [helpful features](https://github.com/CBATeam/CBA_A3/wiki) for mod creators. 

Create an empty directory anywhere on your computer that you want to use, like My Documents. Download HEMTT and place it in the new directory you have just created.

Open up a Terminal on Linux or Command Prompt on Windows and change to your project directory. On Windows you can also use `<Shift> + <Right Click>` and select either `Open command window here` or `Open PowerShell window here`.

**Linux**
```
./hemtt create
```

**Windows**
```
.\hemtt.exe create
```

Follow the prompts to create a new HEMTT Project.
```
Project Name (My Cool Mod): Advanced Banana Environment
Prefix (MCM): ABE3  
Author: ACE Mod Team
Downloading script_macros_common.hpp
```

## Using an Existing Mod

Using an existing project is easy with HEMTT, but some HEMTT utilities may be incompatible with your project. HEMTT only requires that your project uses the following structure.

```
.
├── addons/
    ├── banana/
    └── main/
└── hemtt.json
```

HEMTT will create a pbo for each directory inside the `addons/` folder.

Open up a Terminal on Linux or Command Prompt on Windows and change to your project directory. On Windows you can also use `<Shift> + <Right Click>` and select either `Open command window here` or `Open PowerShell window here`.

**Linux**
```
./hemtt init
```

**Windows**
```
.\hemtt.exe init
```

Follow the prompts to create a new HEMTT Project.
```
Project Name (My Cool Mod): Advanced Banana Environment
Prefix (MCM): ABE3  
Author: ACE Mod Team
Downloading script_macros_common.hpp
```

HEMTT will look for `addons/main/script_version.hpp` and use it for the version number. If you are not using the CBA project structure or do not that file, you can add a version number to the [HEMTT.json Project File](json.md).

```json
{
  "name": "Advanced Banana Environment",
  "prefix": "ABE3",
  "author": "ACE Mod Team",
  "version": "1.0.0.0"
}
```

## Notes

It is recommended you add the following to your `.gitignore` if you are putting your mod on GitHub:
```
*.pbo
*.biprivatekey
releases/
keys/
```
