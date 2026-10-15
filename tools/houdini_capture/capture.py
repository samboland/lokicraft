"""Run declarative Houdini UI scenarios on an isolated Windows desktop."""
import argparse
import ctypes
from ctypes import wintypes as W
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import uuid


class StartupInfo(ctypes.Structure):
    _fields_ = [('cb', W.DWORD), ('lpReserved', W.LPWSTR), ('lpDesktop', W.LPWSTR),
        ('lpTitle', W.LPWSTR), ('dwX', W.DWORD), ('dwY', W.DWORD),
        ('dwXSize', W.DWORD), ('dwYSize', W.DWORD), ('dwXCountChars', W.DWORD),
        ('dwYCountChars', W.DWORD), ('dwFillAttribute', W.DWORD),
        ('dwFlags', W.DWORD), ('wShowWindow', W.WORD), ('cbReserved2', W.WORD),
        ('lpReserved2', ctypes.POINTER(ctypes.c_byte)), ('hStdInput', W.HANDLE),
        ('hStdOutput', W.HANDLE), ('hStdError', W.HANDLE)]


class ProcessInfo(ctypes.Structure):
    _fields_ = [('hProcess', W.HANDLE), ('hThread', W.HANDLE),
               ('dwProcessId', W.DWORD), ('dwThreadId', W.DWORD)]


def read_scenario(path):
    data = json.loads(Path(path).read_text(encoding='utf-8'))
    if not isinstance(data, dict) or not isinstance(data.get('steps'), list):
        raise ValueError('Scenario requires a steps array')
    allowed = {'inspect', 'desktop', 'pane', 'create_node', 'select_node',
               'set_parameters', 'capture', 'load_project', 'export_parameters',
               'capture_shelves', 'show_shelf', 'context_menu', 'floating_pane', 'reference_state'}
    for step in data['steps']:
        if not isinstance(step, dict) or step.get('action') not in allowed:
            raise ValueError(f'Unknown scenario action: {step}')
        delay = step.get('settle_ms', 1000)
        if not isinstance(delay, int) or not 100 <= delay <= 30000:
            raise ValueError('settle_ms must be between 100 and 30000')
        capture_delay = 1000 + 700*len(step['followups']) if 'followups' in step else (2000 if step.get('method') in ['hover', 'tooltip'] else (1000 if 'hover' in step else 600))
        capture_delay = max(capture_delay, step.get('capture_after_ms', 0))
        if step['action'] == 'context_menu' and delay <= capture_delay:
            raise ValueError('Context-menu delay must exceed the popup capture timer')
    return data


def run(args):
    if sys.platform != 'win32':
        raise RuntimeError('This launcher currently supports Windows only')
    scenario = read_scenario(args.scenario)
    exe = Path(args.houdini).resolve(strict=True)
    output = Path(args.output).resolve()
    # Never merge a new run into existing evidence.
    output.mkdir(parents=True, exist_ok=False)
    (output / 'scenario.json').write_text(json.dumps(scenario, indent=2), encoding='utf-8')
    user = ctypes.WinDLL('user32', use_last_error=True)
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    user.CreateDesktopW.argtypes = [W.LPCWSTR, W.LPCWSTR, ctypes.c_void_p, W.DWORD, W.DWORD, ctypes.c_void_p]
    user.CreateDesktopW.restype = W.HANDLE
    user.CloseDesktop.argtypes = [W.HANDLE]
    user.GetForegroundWindow.restype = W.HWND
    kernel.CreateProcessW.argtypes = [W.LPCWSTR, W.LPWSTR, ctypes.c_void_p, ctypes.c_void_p,
        W.BOOL, W.DWORD, ctypes.c_void_p, W.LPCWSTR, ctypes.POINTER(StartupInfo), ctypes.POINTER(ProcessInfo)]
    kernel.CreateProcessW.restype = W.BOOL
    kernel.WaitForSingleObject.argtypes = [W.HANDLE, W.DWORD]
    kernel.TerminateProcess.argtypes = [W.HANDLE, W.UINT]
    kernel.CloseHandle.argtypes = [W.HANDLE]
    kernel.GetExitCodeProcess.argtypes = [W.HANDLE, ctypes.POINTER(W.DWORD)]
    desktop_name = 'LokiCapture_' + uuid.uuid4().hex
    desktop = user.CreateDesktopW(desktop_name, None, None, 0, 0x01FF, None)
    if not desktop:
        raise ctypes.WinError(ctypes.get_last_error())
    env = dict(os.environ)
    env.update(LOKI_CAPTURE_ROOT=str(output), HOUDINI_USER_PREF_DIR=str(output / 'prefs'),
               LOKI_CAPTURE_WORKER_DIR=str(Path(__file__).resolve().parent),
               HOUDINI_NO_START_PAGE='1', HOUDINI_NO_SPLASH='1')
    block = ctypes.create_unicode_buffer('\0'.join(f'{k}={v}' for k,v in sorted(env.items())) + '\0\0')
    command = [str(exe), '-apprentice', f'-geometry={args.width}x{args.height}+0+0',
               'waitforui', str(Path(__file__).with_name('worker.py').resolve())]
    si = StartupInfo()
    si.cb = ctypes.sizeof(si)
    si.lpDesktop = 'winsta0\\' + desktop_name
    pi = ProcessInfo()
    record = {'desktop': desktop_name, 'executable':str(exe), 'command':command,
              'foreground_before':user.GetForegroundWindow(), 'started':time.time(),
              'mode':'isolated-windows-desktop', 'status':'starting'}
    result_path = output / 'result.json'
    status = 1
    try:
        if not kernel.CreateProcessW(str(exe), ctypes.create_unicode_buffer(subprocess.list2cmdline(command)),
                None, None, False, 0x400, block, str(output), ctypes.byref(si), ctypes.byref(pi)):
            raise ctypes.WinError(ctypes.get_last_error())
        record['pid'] = pi.dwProcessId
        print(json.dumps({'output':str(output), 'pid':pi.dwProcessId}), flush=True)
        deadline = time.monotonic() + args.timeout
        while time.monotonic() < deadline:
            if result_path.exists():
                result = json.loads(result_path.read_text(encoding='utf-8'))
                status = 0 if result.get('ok') else 1
                record['status'] = 'complete' if status == 0 else 'scenario-error'
                print(json.dumps({'ok':result.get('ok'),
                    'completed_steps':len(result.get('steps',result.get('completed',[]))),
                    'error':result.get('error'), 'result_file':str(result_path)}), flush=True)
                break
            if kernel.WaitForSingleObject(pi.hProcess, 250) == 0:
                code = W.DWORD()
                kernel.GetExitCodeProcess(pi.hProcess, ctypes.byref(code))
                raise RuntimeError(f'Houdini exited before completing the scenario: {code.value}')
        else:
            raise TimeoutError('Houdini scenario timed out; inspect progress.json and startup.json')
    except KeyboardInterrupt:
        record.update(status='interrupted')
        status = 130
    except Exception as exc:
        record.update(status='failed', error=str(exc))
        print(str(exc), file=sys.stderr)
    finally:
        if pi.hProcess:
            # Only terminate the process created by this launcher. Never attach to user sessions.
            if kernel.WaitForSingleObject(pi.hProcess, 0) != 0:
                kernel.TerminateProcess(pi.hProcess, 0)
                kernel.WaitForSingleObject(pi.hProcess, 5000)
            kernel.CloseHandle(pi.hThread)
            kernel.CloseHandle(pi.hProcess)
        user.CloseDesktop(desktop)
        record.update(finished=time.time(), foreground_after=user.GetForegroundWindow())
        (output / 'launch.json').write_text(json.dumps(record,indent=2), encoding='utf-8')
    return status


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--houdini', required=True, help='Path to houdinifx.exe')
    parser.add_argument('--scenario', required=True)
    parser.add_argument('--output', required=True, help='New directory for this run')
    parser.add_argument('--width', type=int, default=1600)
    parser.add_argument('--height', type=int, default=1000)
    parser.add_argument('--timeout', type=int, default=180)
    args = parser.parse_args()
    if args.width < 640 or args.height < 480 or args.timeout < 1:
        parser.error('Minimum size is 640x480; timeout must be positive')
    return run(args)


if __name__ == '__main__':
    sys.exit(main())
