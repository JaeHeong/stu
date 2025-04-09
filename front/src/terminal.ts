import { Terminal as XTerminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import WebTerminalClient from './WebTerminalClient'; // Import the client

export default class Terminal {
  private readonly xterm: XTerminal;
  private readonly fitAddon = new FitAddon();
  private readonly socketClient: WebTerminalClient; // Add socket client instance
  private container: HTMLDivElement | null = null; // Store container reference

  constructor(socketClient: WebTerminalClient) { // Accept client in constructor
    this.socketClient = socketClient; // Store the client

    this.xterm = new XTerminal({
      cursorBlink: true,
      scrollSensitivity: 2,
      theme: {
        background: '#222',
        foreground: '#ddd', // Lighter foreground for better readability
        cursor: '#f8f8f8', // White cursor
      },
      fontSize: 14, // Set a default font size
      fontFamily: 'monospace', // Use a monospace font
      allowProposedApi: true, // Needed for some addons or future features
    });

    // Load the FitAddon
    this.xterm.loadAddon(this.fitAddon);

    // --- Setup Socket Event Handlers ---
    this.setupSocketEventHandlers();

    // --- Setup Terminal Data Handler ---
    // Handle data typed into the terminal by the user
    this.xterm.onData((data) => {
      // Send the typed data to the server via the socket
      this.socketClient.emitType(data);
    });

     // Handle terminal resize events
     this.xterm.onResize(({ cols, rows }) => {
        // Inform the backend about the resize (Optional: if backend needs it)
        // this.socketClient.emit('resize', { cols, rows });
        console.log(`Terminal resized to ${cols} cols, ${rows} rows`);
     });
  }

  // Method to open the terminal in a given container element
  open(container: HTMLDivElement) {
    this.container = container; // Store container
    this.xterm.open(container); // Attach xterm.js to the container
    this.fit(); // Fit the terminal to the container size
    this.xterm.focus(); // Focus the terminal for immediate input

    // Connect the socket client when the terminal is opened
    this.socketClient.connect();
  }

  // Method to fit the terminal to its container size
  fit() {
     try {
        this.fitAddon.fit();
     } catch (e) {
        console.error("Error fitting terminal:", e);
     }
  }

  // Method to clean up resources when the terminal is no longer needed
  dispose() {
    // Remove socket listeners to prevent memory leaks
    this.socketClient.removeAllListeners();
    // Disconnect the socket
    this.socketClient.disconnect();
    // Dispose of the xterm instance
    this.xterm.dispose();
    this.container = null; // Clear container reference
  }

  // Write data to the terminal
  write(data: string) {
    this.xterm.write(data);
  }

  // Write a line to the terminal
  writeln(line: string) {
    this.xterm.writeln(line);
  }

  // Clear the terminal screen
  clear() {
    this.xterm.clear();
  }

  // --- Private Helper Methods ---

  // Sets up handlers for incoming socket events
  private setupSocketEventHandlers() {
    this.socketClient.onConnect(() => {
      this.writeln('\r\n\x1b[32mConnected to server.\x1b[0m'); // Green connected message
      // Request the backend to start the SSH session
      this.socketClient.emitConnectTerminal();
      this.xterm.focus(); // Ensure terminal is focused after connect
    });

    this.socketClient.onDisconnect(() => {
      this.writeln('\r\n\x1b[31mDisconnected from server.\x1b[0m'); // Red disconnected message
    });

    this.socketClient.onUpdate((data) => {
      // Write data received from the server (SSH output) to the terminal
      this.write(data);
    });

    this.socketClient.onEof(() => {
      this.writeln('\r\n\x1b[33mConnection closed by server (EOF).\x1b[0m'); // Yellow EOF message
      // Optionally disconnect the client here if needed
      // this.socketClient.disconnect();
    });

    this.socketClient.onError((message) => {
      // Write error messages received from the server to the terminal
      this.writeln(`\r\n\x1b[31mServer Error: ${message}\x1b[0m`); // Red error message
    });
  }
}
