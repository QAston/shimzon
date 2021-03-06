## Shimzon

### todo:
- https://docs.microsoft.com/en-us/previous-versions/bb756929(v=msdn.10)?redirectedfrom=MSDN - handle uac better (run mt.exe -manifest "$(ProjectDir)$(TargetName).exe.manifest" -updateresource:"$(TargetDir)$(TargetName).exe;#1") to add manifest to an already existing exe file
- https://stackoverflow.com/questions/224225/create-an-application-without-a-window - handle 
- implement some of the arguments: https://github.com/chocolatey/shimgen#shim-arguments

config.toml
```
base_out_path="C:\\portable\\bin"
shim_exe_path="C:\\portable\\shimzon\\shimexe\\bin\\shim.exe"
```