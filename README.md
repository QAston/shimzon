## Shimzon

A terminal utility that creates an executable shim, which wraps the shimmed application in a manner transparent for the runner.

### Example use cases

1. Selectively adding executables to %PATH% without having to add entire directories which might contain other executables or DLLs
2. Making an alias of an executable which will run the executable with the specified configuration:
    - working directory
    - environment variable
    - arguments
3. Making aliases/wrappers for shell scripts which can run in any windows shell (msys/cygwin bash, cmd, powershell)
4. A workaround for Cygwin/Msys Bash-based shells which [can't handle MSIX's reparse points](https://github.com/msys2/MSYS2-packages/issues/1943)

### Related tools 

- [portable_env](https://github.com/QAston/portable_env) - dynamically update your %PATH% variable in any windows shell (bash, cmd, powershell)

### Alternatives

- <https://github.com/chocolatey/shimgen> - restrictive license, closed source
- <https://github.com/pfmoore/shimmy> - depends on python, embeds executable's config in the binary, fewer config options for target executable
- <https://github.com/lukesampson/shimexe> - supports shims for bat/ps1 files in addition to exe, fewer config options for target executable
- [execution alias feature of MSIX](https://www.tiraniddo.dev/2019/09/overview-of-windows-execution-aliases.html) - limited to applications using MSIX, the path is hardcoded to a single directory, doesn't work in cygin/msys bash
- making a symlink - doesn't work for executables which load dlls from directories of the executable, no config options for the target executable

### Todos

- https://docs.microsoft.com/en-us/previous-versions/bb756929(v=msdn.10)?redirectedfrom=MSDN - handle uac better (run mt.exe -manifest "$(ProjectDir)$(TargetName).exe.manifest" -updateresource:"$(TargetDir)$(TargetName).exe;#1") to add manifest to an already existing exe file
- https://stackoverflow.com/questions/224225/create-an-application-without-a-window - handle 
- implement some of the arguments: https://github.com/chocolatey/shimgen#shim-arguments
- documentation
- error handling
- shimzon sync which reads a .shim file in a directory:
    - .shim specifies where to take executables from, additional options per entry and whether to exclude something from shimming
    - when there's no .shim file it will create one
    - executable.shim file has an additional entry `delete_on_sync=true` which marks the shim for deletion when sync is called
- shimzon update
    - regenerate the executable, keep config, possibly apply config updates 
