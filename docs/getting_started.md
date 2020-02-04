# Getting Started

Create an empty directory anywhere on your computer that you want to use, like My Documents. Download HEMTT and place it in the new directory you have just created. On Windows it is recommended to use the included installer.

Open up a Terminal on Linux or Command Prompt on Windows and change to your project directory. On Windows you can also use `<Shift> + <Right Click>` and select either `Open command window here` or `Open PowerShell window here`.
Using an existing project is easy with HEMTT, but some HEMTT utilities may be incompatible with your project. HEMTT only requires that your project uses the following structure.

```
.
├── addons/
    ├── banana/
    └── main/
└── hemtt.json
```

HEMTT will create a pbo for each directory inside the `addons/` folder.

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
```

HEMTT will look for `addons/main/script_version.hpp` and use it for the version number. If you are not using the CBA project structure or do not that file, you can add a version number to the [HEMTT.json Project File](json.md).

```toml
name: "Advanced Banana Environment"
prefix: "ABE3"
author: "ACE Mod Team"
version: "1.0.0.0"
```

## Notes

It is recommended you add the following to your `.gitignore` if you are putting your mod on GitHub:
```
*.pbo
*.biprivatekey
releases/
keys/
```
