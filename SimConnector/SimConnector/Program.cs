
using System.Runtime.InteropServices;
namespace SimConnector
{
    internal class Program
    {
[   DllImport("kernel32.dll")]
    static extern IntPtr GetConsoleWindow();

    [DllImport("user32.dll")]
    static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")]
    static extern bool SetForegroundWindow(IntPtr hWnd);

        public static void Main(string[] args)
        {

            foreach (var arg in args)
            {
                if (arg == "hide")
                {
                    IntPtr handle = GetConsoleWindow();
                    SetForegroundWindow(handle);
                    ShowWindow(handle, 0);
                }
            }

            // COMPILE:
            // dotnet publish -c Release --self-contained -p:PublishReadyToRun=false -p:PublishTrimmed=true -p:TrimMode=CopyUsed -p:PublishSingleFile=true -p:IncludeAllContentForSelfExtract=true
            SocketCom scket = new SocketCom();
        }
    }
}
