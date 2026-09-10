// wsm-my-lisp-cyberpunk RED4ext plugin -- skeleton per docs/game-injection-plan.md.
//
// Purpose of THIS pass: prove wsm_my_lisp_cyberpunk_dll.dll (dll/, the
// win64-nucleus + reader/eval/ffi crate) actually loads inside the game
// process and its wsm_session_init export is callable -- nothing more.
// No real game-facing functionality (host-primitive registration, RTTI
// hooks) is wired up yet; that is a later, separate step.
//
// Owner go-ahead 2026-09-10 (relayed via the my-lisp-cyberpunk coordination
// session, confirmed directly with the user before starting this file) --
// this is the first code in the repo meant to run inside a third-party
// process (the game), not just as a standalone test binary.

#include <RED4ext/RED4ext.hpp>

#include <windows.h>

#include <string>

namespace
{
// Matches dll/src/ffi.rs's `Session` opaque pointer type exactly: this
// plugin never dereferences it, only passes it back to wsm_session_free.
using Session = void;

using WsmSessionInitFn = Session* (*)();
using WsmSessionFreeFn = void (*)(Session*);

HMODULE g_wsmModule = nullptr;
Session* g_wsmSession = nullptr;

// The plugin's own directory, where the loader placed both this DLL and
// wsm_my_lisp_cyberpunk_dll.dll side by side (RED4ext's own convention --
// see docs/game-injection-plan.md's "(a) The standard, sanctioned path").
std::wstring GetOwnDirectory()
{
    wchar_t path[MAX_PATH] = {};
    HMODULE self = nullptr;
    // GetModuleHandleExW with GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS finds
    // THIS DLL's own module handle from an address inside it (this
    // function), regardless of what the game's own module search path is.
    if (!GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            reinterpret_cast<LPCWSTR>(&GetOwnDirectory),
            &self))
    {
        return L"";
    }
    DWORD length = GetModuleFileNameW(self, path, MAX_PATH);
    if (length == 0 || length == MAX_PATH)
    {
        return L"";
    }
    std::wstring full(path, length);
    auto slash = full.find_last_of(L"\\/");
    return slash == std::wstring::npos ? L"" : full.substr(0, slash + 1);
}
} // namespace

RED4EXT_C_EXPORT bool RED4EXT_CALL Main(RED4ext::v1::PluginHandle aHandle, RED4ext::v1::EMainReason aReason,
                                         const RED4ext::v1::Sdk* aSdk)
{
    switch (aReason)
    {
    case RED4ext::v1::EMainReason::Load:
    {
        auto* logger = aSdk->logger;

        std::wstring dllPath = GetOwnDirectory() + L"wsm_my_lisp_cyberpunk_dll.dll";
        g_wsmModule = LoadLibraryW(dllPath.c_str());
        if (g_wsmModule == nullptr)
        {
            // GetLastError() is deliberately not decoded into a message
            // here -- untested against a real failure case (wrong path,
            // missing MSVC runtime, wrong bitness). A future pass should
            // use FormatMessageW for a readable error, not just the code.
            logger->ErrorF(aHandle, "wsm-my-lisp-cyberpunk-plugin: LoadLibraryW failed, GetLastError=%lu",
                            GetLastError());
            return false;
        }

        auto sessionInit =
            reinterpret_cast<WsmSessionInitFn>(GetProcAddress(g_wsmModule, "wsm_session_init"));
        if (sessionInit == nullptr)
        {
            logger->ErrorF(aHandle, "wsm-my-lisp-cyberpunk-plugin: GetProcAddress(wsm_session_init) failed");
            FreeLibrary(g_wsmModule);
            g_wsmModule = nullptr;
            return false;
        }

        g_wsmSession = sessionInit();
        if (g_wsmSession == nullptr)
        {
            logger->ErrorF(aHandle, "wsm-my-lisp-cyberpunk-plugin: wsm_session_init returned null");
            FreeLibrary(g_wsmModule);
            g_wsmModule = nullptr;
            return false;
        }

        logger->InfoF(aHandle,
                       "wsm-my-lisp-cyberpunk-plugin: wsm_my_lisp_cyberpunk_dll.dll loaded, session=%p", g_wsmSession);
        break;
    }
    case RED4ext::v1::EMainReason::Unload:
    {
        if (g_wsmModule != nullptr)
        {
            if (g_wsmSession != nullptr)
            {
                auto sessionFree =
                    reinterpret_cast<WsmSessionFreeFn>(GetProcAddress(g_wsmModule, "wsm_session_free"));
                if (sessionFree != nullptr)
                {
                    sessionFree(g_wsmSession);
                }
                g_wsmSession = nullptr;
            }
            FreeLibrary(g_wsmModule);
            g_wsmModule = nullptr;
        }
        break;
    }
    }

    return true;
}

RED4EXT_C_EXPORT void RED4EXT_CALL Query(RED4ext::v1::PluginInfo* aInfo)
{
    aInfo->name = L"wsm-my-lisp-cyberpunk";
    aInfo->author = L"juv4uk";
    aInfo->version = RED4EXT_V1_SEMVER(0, 1, 0);
    // RUNTIME_VERSION_INDEPENDENT: this skeleton only proves a DLL loads
    // and one export is callable -- it doesn't touch game RTTI/state, so
    // pinning to a specific game version isn't needed yet, unlike a
    // plugin that actually hooks game functions would require.
    aInfo->runtime = RED4EXT_V1_RUNTIME_VERSION_INDEPENDENT;
    aInfo->sdk = RED4EXT_V1_SDK_VERSION_CURRENT;
}

RED4EXT_C_EXPORT uint32_t RED4EXT_CALL Supports()
{
    return RED4EXT_API_VERSION_1;
}
