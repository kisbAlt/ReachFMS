
namespace SimConnector
{
    internal class Program
    {
        public static void Main()
        {
            // COMPILE:
            // dotnet publish -c Release --self-contained -p:PublishReadyToRun=false -p:PublishTrimmed=true -p:TrimMode=CopyUsed -p:PublishSingleFile=true -p:IncludeAllContentForSelfExtract=true
            SocketCom scket = new SocketCom();
        }
    }
}
