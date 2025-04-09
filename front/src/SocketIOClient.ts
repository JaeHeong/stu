import { io, Socket } from 'socket.io-client';

// Interface for defining event maps
interface IEventsMap {
  [event: string]: any;
}

// Default event map interface
interface IDefaultEventsMap {
  [event: string]: (...args: any[]) => void;
}

// Wrapper class for socket.io-client Socket
class SocketIOClient<
  ListenEvents extends IEventsMap = IDefaultEventsMap,
  EmitEvents extends IEventsMap = ListenEvents,
> {
  readonly client: Socket<ListenEvents, EmitEvents>;

  // Constructor initializes the socket connection
  constructor(namespace: string = '') {
    // Connect to the server, defaulting to localhost:8081
    // Note: The server uses port 8081 for Socket.IO
    this.client = io(`http://localhost:8081${namespace}`, {
      reconnectionAttempts: 5, // Attempt reconnection 5 times
      autoConnect: false, // Do not connect automatically on instantiation
      transports: ['websocket'], // Prefer websocket transport
    });
  }
}

export default SocketIOClient;