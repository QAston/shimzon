#include <corecrt_wstring.h>
#pragma comment(lib, "SHELL32.LIB")
#include <windows.h>
#include <stdio.h>

#include <string>
#include <string_view>
#include <tuple>
#include <optional>
#include <memory>
#include <vector>
#include "cpptoml.h"

#ifndef ERROR_ELEVATION_REQUIRED
#define ERROR_ELEVATION_REQUIRED 740
#endif

BOOL WINAPI CtrlHandler(DWORD ctrlType)
{
    switch (ctrlType)
    {
    // Ignore all events, and let the child process
    // handle them.
    case CTRL_C_EVENT:
    case CTRL_CLOSE_EVENT:
    case CTRL_LOGOFF_EVENT:
    case CTRL_BREAK_EVENT:
    case CTRL_SHUTDOWN_EVENT:
        return TRUE;

    default:
        return FALSE;
    }
}

struct HandleDeleter
{
    typedef HANDLE pointer;
    void operator() (HANDLE handle)
    {
        if (handle)
        {
            CloseHandle(handle);
        }
    }
};

namespace std
{
    typedef unique_ptr<HANDLE, HandleDeleter> unique_handle;
    typedef optional<wstring> wstring_p;
}


std::shared_ptr<cpptoml::table> GetShimInfo()
{
    // Find filename of current executable.
    wchar_t filename[MAX_PATH + 2];
    const auto filenameSize = GetModuleFileNameW(nullptr, filename, MAX_PATH);

    if (filenameSize >= MAX_PATH)
    {
        fprintf(stderr, "The filename of the program is too long to handle.\n");
        return {};
    }

    // Use filename of current executable to find .shim
    wmemcpy(filename + filenameSize - 3, L"shim", 4U);
    filename[filenameSize + 1] = L'\0';
    FILE* fp = nullptr;

    if (_wfopen_s(&fp, filename, L"r,ccs=UTF-8") != 0)
    {
        fprintf(stderr, "Cannot open shim file for read.\n");
        return {};
    }

    std::wstring ws(filename);
    // your new String
    std::string str{ws.begin(), ws.end()};

    return cpptoml::parse_file(str);
}

std::tuple<std::unique_handle, std::unique_handle> MakeProcess(const std::wstring_p& path, const std::wstring_p& args)
{
    // Start subprocess
    STARTUPINFOW si = {};
    PROCESS_INFORMATION pi = {};

    std::vector<wchar_t> cmd(path->size() + args->size() + 2);
    wmemcpy(cmd.data(), path->c_str(), path->size());
    cmd[path->size()] = L' ';
    wmemcpy(cmd.data() + path->size() + 1, args->c_str(), args->size());
    cmd[path->size() + 1 + args->size()] = L'\0';

    std::unique_handle threadHandle;
    std::unique_handle processHandle;

    if (CreateProcessW(nullptr, cmd.data(), nullptr, nullptr, TRUE, CREATE_SUSPENDED, nullptr, nullptr, &si, &pi))
    {
        threadHandle.reset(pi.hThread);
        processHandle.reset(pi.hProcess);

        ResumeThread(threadHandle.get());
    }
    else
    {
        if (GetLastError() == ERROR_ELEVATION_REQUIRED)
        {
            // We must elevate the process, which is (basically) impossible with
            // CreateProcess, and therefore we fallback to ShellExecuteEx,
            // which CAN create elevated processes, at the cost of opening a new separate
            // window.
            // Theorically, this could be fixed (or rather, worked around) using pipes
            // and IPC, but... this is a question for another day.
            SHELLEXECUTEINFOW sei = {};

            sei.cbSize = sizeof(SHELLEXECUTEINFOW);
            sei.fMask = SEE_MASK_NOCLOSEPROCESS;
            sei.lpFile = path->c_str();
            sei.lpParameters = args->c_str();
            sei.nShow = SW_SHOW;

            if (!ShellExecuteExW(&sei))
            {
                fprintf(stderr, "Unable to create elevated process: error %li.", GetLastError());
                return {std::move(processHandle), std::move(threadHandle)};
            }

            processHandle.reset(sei.hProcess);
        }
        else
        {
            fprintf(stderr, "Could not create process with command '%ls'.\n", cmd.data());
            return {std::move(processHandle), std::move(threadHandle)};
        }
    }

    // Ignore Ctrl-C and other signals
    if (!SetConsoleCtrlHandler(CtrlHandler, TRUE))
    {
        fprintf(stderr, "Could not set control handler; Ctrl-C behavior may be invalid.\n");
    }

    return {std::move(processHandle), std::move(threadHandle)};
}

template <typename T>
std::wstring_p to_wchar(const cpptoml::option<T>& val) {
    if (val) {
        return { std::wstring{val->begin(), val->end()}};
    } else {
        return {};
    }
}

std::wstring str_to_wchar(const std::string& val) {
    return std::wstring{val.begin(), val.end()};
}

int wmain(int argc, wchar_t* argv[])
{
    auto table = GetShimInfo();

    if (!table)
    {
        fprintf(stderr, "Could not read .shim file.\n");
        return 1;
    }

    auto path = to_wchar(table->get_as<std::string>("path"));
    
    if (!path)
    {
        fprintf(stderr, "Could not find path in .shim file.\n");
        return 1;
    }

    auto args = to_wchar(table->get_as<std::string>("args"));
    if (!args)
    {
        args.emplace();
    }

    auto cmd = GetCommandLineW();
    if (cmd[0] == L'\"')
    {
        args->append(cmd + wcslen(argv[0]) + 2);
    }
    else
    {
        args->append(cmd + wcslen(argv[0]));
    }

    if (std::shared_ptr<cpptoml::table> env_table = table->get_table("env")) {
        for (const auto &entry : *env_table) {
            auto key = str_to_wchar(entry.first);
            auto value = to_wchar(env_table->get_as<std::string>(entry.first));
            SetEnvironmentVariableW(key.c_str(), value->c_str());
        }
    }

    // Find out if the target program is a console app
    SHFILEINFOW sfi = {};
    const auto isWindowsApp = HIWORD(SHGetFileInfoW(path->c_str(), -1, &sfi, sizeof(sfi), SHGFI_EXETYPE)) != 0;

    if (isWindowsApp)
    {
        // Unfortunately, this technique will still show a window for a fraction of time,
        // but there's just no workaround.
        FreeConsole();
    }

    // Create job object, which can be attached to child processes
    // to make sure they terminate when the parent terminates as well.
    std::unique_handle jobHandle(CreateJobObject(nullptr, nullptr));
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION jeli = {};

    jeli.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK;
    SetInformationJobObject(jobHandle.get(), JobObjectExtendedLimitInformation, &jeli, sizeof(jeli));

    auto [processHandle, threadHandle] = MakeProcess(std::move(path), std::move(args));
    if (processHandle && !isWindowsApp)
    {
        AssignProcessToJobObject(jobHandle.get(), processHandle.get());

        // Wait till end of process
        WaitForSingleObject(processHandle.get(), INFINITE);

        DWORD exitCode = 0;
        GetExitCodeProcess(processHandle.get(), &exitCode);

        return exitCode;
    }

    return processHandle ? 0 : 1;
}
