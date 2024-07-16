

using System.Runtime.InteropServices;
using System.Text;

namespace SimConnector
{
    internal class ParentWatcher
    {
        // Import necessary functions from Psapi.dll
        [DllImport("psapi.dll", SetLastError = true)]
        public static extern bool EnumProcesses([MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 1)][In] uint[] lpidProcess, uint cb, [MarshalAs(UnmanagedType.U4)] out uint lpcbNeeded);

        [DllImport("psapi.dll", SetLastError = true, CharSet = CharSet.Auto)]
        public static extern bool EnumProcessModules(IntPtr hProcess, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 2)][Out] IntPtr[] lphModule, uint cb, [MarshalAs(UnmanagedType.U4)] out uint lpcbNeeded);

        [DllImport("psapi.dll", SetLastError = true, CharSet = CharSet.Auto)]
        public static extern uint GetModuleBaseName(IntPtr hProcess, IntPtr hModule, [Out] StringBuilder lpBaseName, uint nSize);

        // Import necessary functions from kernel32.dll
        [DllImport("kernel32.dll", SetLastError = true)]
        public static extern IntPtr OpenProcess(uint processAccess, bool bInheritHandle, uint processId);

        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        public static extern bool CloseHandle(IntPtr hObject);

        // Constants
        const uint PROCESS_QUERY_INFORMATION = 0x0400;
        const uint PROCESS_VM_READ = 0x0010;
        const string _parent_process = "reachfms";
        public static bool ParentRunning()
        {
            return IsProcessRunning(_parent_process);
        }

        static bool IsProcessRunning(string processName)
        {
            uint[] processIds = new uint[1024];
            uint bytesNeeded;

            if (!EnumProcesses(processIds, (uint)processIds.Length * sizeof(uint), out bytesNeeded))
            {
                return false;
            }

            int processCount = (int)(bytesNeeded / sizeof(uint));

            for (int i = 0; i < processCount; i++)
            {
                IntPtr hProcess = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, processIds[i]);
                if (hProcess != IntPtr.Zero)
                {
                    StringBuilder baseName = new StringBuilder(1024);
                    if (EnumProcessModules(hProcess, new IntPtr[1], (uint)IntPtr.Size, out uint cbNeeded))
                    {
                        if (GetModuleBaseName(hProcess, IntPtr.Zero, baseName, (uint)baseName.Capacity) > 0)
                        {
                            if (baseName.ToString().Equals($"{processName}.exe", StringComparison.OrdinalIgnoreCase))
                            {
                                CloseHandle(hProcess);
                                return true;
                            }
                        }
                    }
                    CloseHandle(hProcess);
                }
            }
            return false;
        }
    }
}
