## Shimzon

### todo:
- https://docs.microsoft.com/en-us/previous-versions/bb756929(v=msdn.10)?redirectedfrom=MSDN - handle uac better (run mt.exe -manifest "$(ProjectDir)$(TargetName).exe.manifest" -updateresource:"$(TargetDir)$(TargetName).exe;#1") to add manifest to an already existing exe file
- https://stackoverflow.com/questions/224225/create-an-application-without-a-window - handle 
- implement some of the arguments: https://github.com/chocolatey/shimgen#shim-arguments
- documentation
- error handling
- better config
- shimzon sync which reads a .shim file in a directory:
    - .shim specifies where to take executables from, additional options per entry and whether to exclude something from shimming
    - when there's no .shim file it will create one
    - executable.shim file has an additional entry `delete_on_sync=true` which marks the shim for deletion when sync is called
- shimzon update
    - regenerate the executable, keep config, possibly apply config updates 

config.toml
```
base_out_path="C:\\portable\\bin"
shim_exe_path="C:\\portable\\shimzon\\shimexe\\bin\\shim.exe"
```
