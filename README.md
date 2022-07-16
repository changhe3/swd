# swd
swd 0.1.1

(This software has not been extensively tested, use at your own risk. Require steamcmd under PATH)

You would need [SteamCMD](https://developer.valvesoftware.com/wiki/SteamCMD#Downloading_SteamCMD) to use this software,
and remember to include it under the PATH environment variable. A command-line utility to download workshop item and
collections from steam workshop. This software assembles a command for SteamCMD to execute. By default, this command is
only printed to standard output, you need to use the `-e` flag to automatically execute the command. The default
download directory is /path/to/steamcmd/steamapps/workshop/content/. You can set an alternative location with `-o`.


```
USAGE:
    swd [FLAGS] [OPTIONS] [files]...

FLAGS:
    -e, --exec       
            Execute the produced command through steamcmd, otherwise the command is only printed to standard output and
            need to be executed manually
    -h, --help       
            Prints help information

    -r, --review     
            Review each mod one by one. Input yes/no/skip for each mod or collection. The option 'skip', otherwise
            equivalent to 'no', can be used to skip rest of the mods in the context of a collection
    -V, --version    
            Prints version information


OPTIONS:
        --save=<format>          
            Save the mod orders of collections to specified format to the current working directory [possible values:
            simple, csv]
    -o, --output <path>          
            Set the path of the download location. The path will be passed to force_install_dir in SteamCMD

    -u, --username <username>    
            Steam username for non-anonymous download [default: anonymous]


ARGS:
    <files>...    
            File IDs of the mods and collections to download, can be found at the end of the url for each workshop item
```