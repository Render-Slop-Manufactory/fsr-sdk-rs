// SPDX-License-Identifier: MPL-2.0
// Bounded Windows heartbeat fixture; no GPU calls or deliberate device hangs.
#include <windows.h>
#include <cstdio>
#include <string>

int main(int argc, char** argv) {
    if (argc != 3) return 2;
    const std::string mode = argv[1], base = argv[2];
    if (mode == "once") return 0;
    if (mode == "parent") {
        char executable[MAX_PATH] = {};
        if (!GetModuleFileNameA(nullptr, executable, MAX_PATH)) return 3;
        std::string command = std::string("\"") + executable + "\" child \"" + base + "\"";
        STARTUPINFOA startup = {};
        startup.cb = sizeof(startup);
        PROCESS_INFORMATION process = {};
        if (!CreateProcessA(nullptr, command.data(), nullptr, nullptr, FALSE,
                            DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP, nullptr,
                            nullptr, &startup, &process)) return 4;
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
    }
    const auto heartbeat = base + "." + mode;
    const auto stop = base + ".stop";
    FILE* stream = nullptr;
    if (fopen_s(&stream, heartbeat.c_str(), "ab") || !stream) return 5;
    // Cooperative stop and hard lifetime bounds keep even a failed watchdog probe finite.
    const auto deadline = GetTickCount64() + 120000;
    while (GetTickCount64() < deadline && GetFileAttributesA(stop.c_str()) == INVALID_FILE_ATTRIBUTES) {
        std::fputc('.', stream);
        std::fflush(stream);
        Sleep(100);
    }
    std::fclose(stream);
    return 0;
}
