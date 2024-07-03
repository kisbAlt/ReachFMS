using System.Net.WebSockets;
using System.Text;


namespace SimConnector
{
    internal class SocketCom
    {
        WsClient client;
        const string _ws_addr = "ws://localhost:5273/ws/";
        public SocketCom()
        {
            WasmConnect wasm = new WasmConnect();

            this.client = new WsClient(wasm);
            this.TryConnecting();
        }

        public async void TryConnecting()
        {
            await client.ConnectAsync(_ws_addr);
            Console.WriteLine($"Connected: {client.Connected}");
            Thread.Sleep(50);
            if (client.WSConnected)
            {
                await client.SendMessageAsync("ConnectWSClient");
            }else
            {
                Console.WriteLine("Can't connect to client...");
            }
        }
    }

    public class WsClient : IDisposable
    {
        public bool Connected = false;
        public int ReceiveBufferSize { get; set; } = 8192;
        const int bufferSize = 1024;
        readonly byte[] receiveBuffer = new byte[bufferSize];
        readonly byte[] sendBuffer = new byte[bufferSize];
        private ClientWebSocket WS;
        private CancellationTokenSource CTS;
        public WasmConnect wasm;
        public bool WSConnected = true;

        public WsClient(WasmConnect wasm)
        {
            this.wasm = wasm;
        }


        public async Task ConnectAsync(string url)
        {
            if (WS != null)
            {
                if (WS.State == WebSocketState.Open) return;
                else WS.Dispose();
            }
            WS = new ClientWebSocket();
            if (CTS != null) CTS.Dispose();
            CTS = new CancellationTokenSource();
            await WS.ConnectAsync(new Uri(url), CTS.Token);
            await Task.Factory.StartNew(ReceiveLoop, CTS.Token, TaskCreationOptions.LongRunning, TaskScheduler.Default);
        }

        public async Task DisconnectAsync()
        {
            if (WS is null) return;
            // TODO: requests cleanup code, sub-protocol dependent.
            if (WS.State == WebSocketState.Open)
            {
                CTS.CancelAfter(TimeSpan.FromSeconds(2));
                await WS.CloseOutputAsync(WebSocketCloseStatus.Empty, "", CancellationToken.None);
                await WS.CloseAsync(WebSocketCloseStatus.NormalClosure, "", CancellationToken.None);
            }
            WS.Dispose();
            WS = null;
            CTS.Dispose();
            CTS = null;
        }

        private async Task ReceiveLoop()
        {
            var loopToken = CTS.Token;
            MemoryStream outputStream = null;
            WebSocketReceiveResult receiveResult = null;
            var buffer = new byte[ReceiveBufferSize];
            try
            {
                while (!loopToken.IsCancellationRequested)
                {
                    outputStream = new MemoryStream(ReceiveBufferSize);
                    do
                    {
                        receiveResult = await WS.ReceiveAsync(buffer, CTS.Token);
                        if (receiveResult.MessageType != WebSocketMessageType.Close)
                            outputStream.Write(buffer, 0, receiveResult.Count);
                    }
                    while (!receiveResult.EndOfMessage);
                    if (receiveResult.MessageType == WebSocketMessageType.Close) break;
                    outputStream.Position = 0;
                    ResponseReceived(outputStream);
                }
            }
            catch (TaskCanceledException) { }
            finally
            {
                outputStream?.Dispose();
            }
        }

        public async Task SendMessageAsync(string message)
        {
            var messageLength = message.Length;
            var messageCount = (int)Math.Ceiling((double)messageLength / bufferSize);
            for (var i = 0; i < messageCount; i++)
            {
                var offset = bufferSize * i;
                var count = bufferSize;
                var lastMessage = i + 1 == messageCount;
                if (count * (i + 1) > messageLength)
                    count = messageLength - offset;
                var segmentLength = Encoding.UTF8.GetBytes(message, offset, count, sendBuffer, 0);
                var segment = new ArraySegment<byte>(sendBuffer, 0, segmentLength);
                await WS.SendAsync(segment, WebSocketMessageType.Text, lastMessage, CTS.Token);
            }
        }

        private void ResponseReceived(Stream inputStream)
        {
            StreamReader reader = new StreamReader(inputStream);
            string text = reader.ReadToEnd();
            // protocol examples: 
            // CONNECTED => rust app sent back the comfirmation after first request
            // CMD_BTN:EXAMPLE_LVAR => pressing and releasing EXAMPLE_LVAR lvar
            // CMD_PRESS:EXAMPLE_LVAR => pressing EXAMPLE_LVAR lvar
            // CMD_REL:EXAMPLE_LVAR => pressing EXAMPLE_LVAR lvar
            // CLOSE => Disconnecting and closing the app...
            // RECONNECT => try reconnecting to simconnect
            // STATUS => send bridge status

            //SENDING:
            // STATUS:TRUE => simconnect is connected

            if(text == "CONNECTED")
            {
                Console.WriteLine("RUST APP CONNECTED");
                this.Connected = true;
            }else if (text == "CLOSE")
            {
                wasm.Disconnect();
                Console.WriteLine("TERMINATING SIMCONNECTOR");
                Environment.Exit(0);
            } else if (text == "STATUS")
            {
                this.SendMessageAsync("STATUS:"+wasm.WasmConnected.ToString().ToUpper());
            }
            else if (text.Contains("CMD_BTN"))
            {
                wasm.ButtonPressL(text.Split(":").ElementAt(1));
            }
            else if (text.Contains("GET_AIRCRAFT"))
            {
                this.SendMessageAsync("AIRCRAFT:" + wasm.AircraftTitle.ToUpper());
            }
            Console.WriteLine($"GOT RESP:{text}");
        }

        public void Dispose() => DisconnectAsync().Wait();

    }
}
