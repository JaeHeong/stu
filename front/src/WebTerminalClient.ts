import SocketIOClient from './SocketIOClient';

// Define events the client will listen for from the server
interface IListenEvents {
  update: (data: string) => void; // Server sends terminal output
  eof: () => void; // Server signals connection end
  error: (message: string) => void; // Server sends an error message
}

// Define events the client will emit to the server
interface IEmitEvents {
  connectTerminal: () => void; // Client requests to start the terminal session
  type: (data: string) => void; // Client sends user input
}

// Client specifically for the Web Terminal, extending the base SocketIOClient
class WebTerminalClient extends SocketIOClient<IListenEvents, IEmitEvents> {
  constructor() {
    // Connect to the '/ssh' namespace used by the server
    super('/ssh');

    // Log reconnection failures
    this.client.io.on('reconnect_failed', () => {
      console.error('Socket.IO: Reconnect failed');
      // Optionally notify the user or attempt manual reconnection
    });

    // Log connection errors
    this.client.on('connect_error', (err) => {
      console.error('Socket.IO: Connection error:', err.message);
      // Optionally display an error message in the UI
    });
  }

  // Method to explicitly connect to the server
  connect() {
    if (!this.client.connected) {
      this.client.connect();
    }
  }

  // Method to explicitly disconnect from the server
  disconnect() {
    if (this.client.connected) {
      this.client.disconnect();
    }
  }

  // --- Event Emitters ---

  // Emit event to request terminal session start
  emitConnectTerminal() {
    this.client.emit('connectTerminal');
  }

  // Emit event for user input
  emitType(data: string) {
    this.client.emit('type', data);
  }

  // --- Event Listeners ---

  // Register callback for 'connect' event
  onConnect(callback: () => void) {
    this.client.on('connect', callback);
  }

  // Register callback for 'disconnect' event
  onDisconnect(callback: () => void) {
    this.client.on('disconnect', callback);
  }

  // Register callback for 'update' event (server output)
  onUpdate(callback: (data: string) => void) {
    this.client.on('update', callback);
  }

  // Register callback for 'eof' event (server closed connection)
  onEof(callback: () => void) {
    this.client.on('eof', callback);
  }

  // Register callback for 'error' event
  onError(callback: (message: string) => void) {
    this.client.on('error', callback);
  }

  // Remove all listeners (useful during cleanup)
  removeAllListeners() {
    this.client.offAny();
  }
}

export default WebTerminalClient;