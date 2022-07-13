# swd

```
swd 0.1.0
Download workshop item and collections from steam workshop

USAGE:
    swd [FLAGS] [OPTIONS] [files]...

FLAGS:
    -e, --exec
            Execute the produced command through steamcmd, otherwise the command is only printed to stdout

    -h, --help
            Prints help information

    -r, --review
            Review each mod one by one

    -V, --version
            Prints version information


OPTIONS:
        --save=<save>
            Save the mod orders of collections to specified format to the current working directory [possible values:
            simple, csv]
    -u, --username <username>
             [default: anonymous]


ARGS:
    <files>...
            File IDs of the mods and collections to download
```