Cloister solves the problem of manually having to add files not in your git ignore to a zip file.

## Example

```shell
cloister zip somedirectory 
# somedirectory.zip should now be in the current directory.
```

## Usage

```
Zip non-git ignored files in a directory

Usage: cloister <COMMAND>

Commands:
  zip   Recursively adds all files in <directory> to <directory>.zip. Respects .gitignore files
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```