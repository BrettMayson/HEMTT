# Arma 3 Tools

HEMTT will use your installation of Arma 3 Tools to binarize supported files (p3d, rtm, wrp).

## Installation

### Windows

Arma 3 Tools can be installed using [Steam](https://store.steampowered.com/app/233800/Arma_3_Tools/). After installation, run the tools at least once to ensure the registry keys are set.

### Linux

HEMTT can use either Proton or Wine to run the tools. `wine` or `wine64` is highly recommended, as using Proton will be much slower and may cause windows to pop up while running the tools. HEMTT will always use `wine64` or `wine` if they are available.

#### Steam

Arma 3 Tools can be installed using [Steam](https://store.steampowered.com/app/233800/Arma_3_Tools/) with Proton. You can also use [SteamCMD](https://developer.valvesoftware.com/wiki/SteamCMD) to install the tools on servers.

#### ~/.local/share/arma3tools

The tools can be installed manually into `~/.local/share/arma3tools` by copying the files from a Windows installation. If the tools are installed with Steam and inside this directory, HEMTT will prefer to use `~/.local/share/armatools`.

#### HEMTT_BI_TOOLS Environment Variable

If you have the tools installed in a different location, you can set the `HEMTT_BI_TOOLS` environment variable to the path of the tools. HEMTT will always use this path if it is set.
