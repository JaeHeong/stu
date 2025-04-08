import React, { useEffect, useRef, useState, useCallback } from 'react';
import Terminal from './terminal'; // Assuming Terminal.ts is in the same directory
import WebTerminalClient from './WebTerminalClient'; // Import the client
import '@xterm/xterm/css/xterm.css'; // Import xterm CSS

function App() {
  // Ref to hold the container div element for the terminal
  const containerRef = useRef<HTMLDivElement>(null);
  // Ref to hold the Terminal class instance
  const terminalInstanceRef = useRef<Terminal | null>(null);
  // Ref to hold the WebTerminalClient instance
  const socketClientRef = useRef<WebTerminalClient | null>(null);
  // State to track connection status (optional but good for UI feedback)
  const [isConnected, setIsConnected] = useState(false);

  // Initialize Socket Client only once
  if (!socketClientRef.current) {
    socketClientRef.current = new WebTerminalClient();
  }

  // Effect to setup and cleanup the terminal
  useEffect(() => {
    const container = containerRef.current;
    const socketClient = socketClientRef.current;

    // Ensure container and socket client exist
    if (container && socketClient) {
      // Create a new Terminal instance, passing the socket client
      const terminal = new Terminal(socketClient);
      terminalInstanceRef.current = terminal; // Store instance in ref

      // Open the terminal in the container
      terminal.open(container);

      // --- Handle Resizing ---
      // Create a ResizeObserver to watch the container element
      const resizeObserver = new ResizeObserver(() => {
         // Use requestAnimationFrame to avoid layout thrashing and ensure fit runs smoothly
         window.requestAnimationFrame(() => {
            try {
               terminal.fit();
            } catch (e) {
               console.log("Could not fit terminal:", e);
               // If fit fails (e.g., element not visible), disconnect observer maybe?
            }
         });
      });
      // Start observing the container element
      resizeObserver.observe(container);


      // --- Update Connection State ---
      const handleConnect = () => setIsConnected(true);
      const handleDisconnect = () => setIsConnected(false);
      socketClient.onConnect(handleConnect);
      socketClient.onDisconnect(handleDisconnect);

      // --- Cleanup function ---
      return () => {
        // Stop observing the container
        resizeObserver.disconnect();
        // Dispose of the terminal instance (this also disconnects socket)
        terminal.dispose();
        terminalInstanceRef.current = null; // Clear the ref
        setIsConnected(false); // Reset connection state
        // Explicitly remove listeners set here
        socketClient.client.off('connect', handleConnect);
        socketClient.client.off('disconnect', handleDisconnect);
      };
    }
    // Dependency array: runs only when containerRef changes (effectively on mount)
  }, []); // Empty dependency array ensures this runs only once on mount


  // Callback for manual connect/disconnect button (optional)
  const toggleConnection = useCallback(() => {
    const client = socketClientRef.current;
    if (client) {
      if (client.client.connected) {
        client.disconnect();
      } else {
        client.connect();
      }
    }
  }, []);

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-white">
       {/* Optional Header */}
       <header className="bg-gray-800 p-2 shadow-md flex justify-between items-center">
         <h1 className="text-lg font-semibold">React Web Terminal</h1>
         <button
            onClick={toggleConnection}
            className={`px-3 py-1 rounded ${
              isConnected
                ? 'bg-red-600 hover:bg-red-700'
                : 'bg-green-600 hover:bg-green-700'
            } text-white transition-colors duration-150`}
          >
            {isConnected ? 'Disconnect' : 'Connect'}
          </button>
       </header>

       {/* Terminal Container */}
       {/* The ref is attached here, and xterm.js will render inside this div */}
       <div
         ref={containerRef}
         className="flex-grow p-2 overflow-hidden" // flex-grow makes it take remaining space
         id="terminal-container" // Added ID for clarity
       />
    </div>
  );
}

export default App;