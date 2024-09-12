# hemtt utils sqf case

```admonish danger
This command requires **manual review**. It can have lots of false positives so you are **strongly encouraged** to check each modified file to ensure it is correct.
```

<pre><code>Fix capitalization in SQF commands

Usage: hemtt utils sqf [OPTIONS] &lt;path&gt;

Arguments:
  &lt;path&gt;
          Path to the SQF file or a folder to recursively fix

Options:
    -t, --threads
        Number of threads, defaults to # of CPUs

    -h, --help
        Print help information (use `-h` for a summary)
</code>
</pre>

This will recursively correct all capitalization mistakes in SQF commands.

## Example

```sqf
private _positionASL = GetPosasl Player;
// becomes
private _positionASL = getPosASL player;
```

## False Positives

This command does not full parse your SQF files.

It will not change words in strings in comments, but it may change words that will break your program

```sqf
// script_macros.hpp
#define FALSE 0
#define TRUE 1

// fnc_someFunction.sqf
if (getNumber (configFile >> "someClass" >= TRUE)) then {...};
// becomes
if (getNumber (configFile >> "someClass" >= true)) then {...};
```

```sqf
private _value = player getVariable [QGVAR(showHud), false];
// becomes
private _value = player getVariable [QGVAR(showHUD), false];
```
