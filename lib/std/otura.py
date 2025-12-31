# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ÒTÚRÁ - THE MESSENGER (1011)                                       ║
║                    Network Operations                                        ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  The domain of communication between Opon instances.                         ║
║  Supports both direct TCP sockets and UDP broadcast (Ether network).         ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import socket
import json
import threading
from typing import Any, Callable, Dict, List, Optional

from .base import OduModule


class OturaDomain(OduModule):
    """The Messenger - Network operations using real sockets."""
    
    # Default Ether network settings
    ETHER_PORT = 9256  # Default port for Ether broadcast
    ETHER_GROUP = '239.255.255.250'  # Multicast group for Ether
    
    def __init__(self, name: str = "Orunmila"):
        super().__init__("Òtúrá", "1011", "The Messenger - Network")
        
        # Instance identification
        self.name = name
        
        # TCP Socket state
        self._socket: Optional[socket.socket] = None
        self._client_socket: Optional[socket.socket] = None
        self._listening = False
        
        # Ether (UDP Multicast) state
        self._ether_socket: Optional[socket.socket] = None
        self._ether_recv_socket: Optional[socket.socket] = None
        self._ether_channel = 0
        self._ether_running = False
        self._ether_messages: List[Dict] = []
        self._ether_thread: Optional[threading.Thread] = None
        
        # Callbacks for async receive
        self._on_message: Optional[Callable] = None
        
        # High-level API
        self._register("de", self.de, "Bind to port (TCP)")
        self._register("so", self.so, "TCP Connect to host (different from Ika.so=concat, Obara.so=multiply)")
        self._register("ran", self.ran, "Send data (TCP)")
        self._register("gba", self.gba, "TCP Receive data (different from Ogbe.gba=args, Ogunda.gba=pop)")
        self._register("pa", self.pa, "Close connection")
        self._register("tẹtisi", self.tetisi, "Accept incoming connection")
        
        # Ether (Real UDP Multicast) API
        self._register("ether_de", self.ether_de, "Join Ether network")
        self._register("ether_ran", self.ether_ran, "Broadcast to Ether")
        self._register("ether_gba", self.ether_gba, "Receive from Ether")
        self._register("ether_pa", self.ether_pa, "Leave Ether network")
        
        # Spec Functions
        self._register("gbo", self.gbo, "Listen/Receive (Alias)")
        self._register("so_po", self.so_po, "Connect (Alias)")
        
        # VM-style opcodes
        self.OPCODES = {
            "BIND": "10111011",
            "SEND": "10111111",
            "RECV": "10110000",
        }
    
    # =========================================================================
    # TCP SOCKET API
    # =========================================================================
    
    def de(self, port: int, host: str = '127.0.0.1') -> bool:
        """Bind to port and listen for TCP connections.
        
        Args:
            port: Port number to listen on
            host: Host to bind to. Defaults to '127.0.0.1' (localhost only) for security.
                  Use '0.0.0.0' explicitly to bind to all interfaces.
        """
        print(f"[Òtúrá] {self.name} binding to {host}:{port}...")
        try:
            self._socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self._socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            self._socket.bind((host, port))
            self._socket.listen(5)
            self._listening = True
            print(f"[Òtúrá] {self.name} listening on port {port}")
            return True
        except Exception as e:
            print(f"[Òtúrá] Bind error: {e}")
            return False
    
    def tetisi(self, timeout: float = None) -> bool:
        """Accept an incoming connection. Returns True if client connected."""
        if not self._socket or not self._listening:
            print(f"[Òtúrá] Error: Not listening")
            return False
        
        try:
            if timeout:
                self._socket.settimeout(timeout)
            self._client_socket, addr = self._socket.accept()
            print(f"[Òtúrá] {self.name} accepted connection from {addr}")
            return True
        except socket.timeout:
            return False
        except Exception as e:
            print(f"[Òtúrá] Accept error: {e}")
            return False
    
    def so(self, host: str, port: int) -> bool:
        """Connect to remote host via TCP."""
        print(f"[Òtúrá] {self.name} connecting to {host}:{port}...")
        try:
            self._socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self._socket.connect((host, port))
            print(f"[Òtúrá] {self.name} connected to {host}:{port}")
            return True
        except Exception as e:
            print(f"[Òtúrá] Connect error: {e}")
            return False
    
    def ran(self, data: str) -> bool:
        """Send data over TCP socket."""
        target = self._client_socket or self._socket
        if target:
            try:
                target.send(data.encode('utf-8'))
                print(f"[Òtúrá] {self.name} sent: {data}")
                return True
            except Exception as e:
                print(f"[Òtúrá] Send error: {e}")
        else:
            print(f"[Òtúrá] Error: No active connection")
        return False
    
    def gba(self, buffer_size: int = 4096, timeout: float = None) -> str:
        """Receive data from TCP socket."""
        target = self._client_socket or self._socket
        if target:
            try:
                if timeout:
                    target.settimeout(timeout)
                data = target.recv(buffer_size)
                received = data.decode('utf-8')
                print(f"[Òtúrá] {self.name} received: {received}")
                return received
            except socket.timeout:
                return ""
            except Exception as e:
                print(f"[Òtúrá] Receive error: {e}")
        return ""
    
    def pa(self):
        """Close all connections."""
        had_connection = self._client_socket or self._socket
        if self._client_socket:
            self._client_socket.close()
            self._client_socket = None
        if self._socket:
            self._socket.close()
            self._socket = None
        self._listening = False
        if had_connection:
            print(f"[Òtúrá] {self.name} TCP connection closed.")
    
    # =========================================================================
    # ETHER NETWORK (Real UDP Multicast)
    # =========================================================================
    
    def ether_de(self, channel: int = 0, port: int = None) -> bool:
        """
        Join the Ether network using UDP multicast.
        Channel determines the multicast group (239.255.255.X where X = channel).
        """
        self._ether_channel = channel
        ether_port = port or self.ETHER_PORT
        multicast_group = f"239.255.255.{channel % 256}"
        
        print(f"[Òtúrá] {self.name} joining Ether network (channel {channel})...")
        
        try:
            # Create UDP socket for receiving
            self._ether_recv_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            self._ether_recv_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            
            # Bind to the Ether port (localhost only for security)
            self._ether_recv_socket.bind(('127.0.0.1', ether_port))
            
            # Join multicast group
            import struct
            mreq = struct.pack("4sl", socket.inet_aton(multicast_group), socket.INADDR_ANY)
            self._ether_recv_socket.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, mreq)
            self._ether_recv_socket.settimeout(0.1)
            
            # Create UDP socket for sending
            self._ether_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            self._ether_socket.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 2)
            
            self._ether_running = True
            self._listening = True
            
            # Start background receiver thread
            self._ether_thread = threading.Thread(target=self._ether_receiver, daemon=True)
            self._ether_thread.start()
            
            print(f"[Òtúrá] {self.name} is now listening on Ether channel {channel}")
            return True
            
        except Exception as e:
            print(f"[Òtúrá] Ether join error: {e}")
            return False
    
    def _ether_receiver(self):
        """Background thread to receive Ether messages."""
        multicast_group = f"239.255.255.{self._ether_channel % 256}"
        
        while self._ether_running:
            try:
                data, addr = self._ether_recv_socket.recvfrom(4096)
                message = json.loads(data.decode('utf-8'))
                
                # Don't echo our own messages
                if message.get("from") != self.name:
                    self._ether_messages.append(message)
                    if self._on_message:
                        self._on_message(message)
                        
            except socket.timeout:
                continue
            except Exception:
                continue
    
    def ether_ran(self, value: Any) -> bool:
        """Broadcast a message to the Ether network."""
        if not self._ether_socket:
            print(f"[Òtúrá] Error: Not connected to Ether network")
            return False
        
        multicast_group = f"239.255.255.{self._ether_channel % 256}"
        
        try:
            packet = {
                "from": self.name,
                "val": value,
                "channel": self._ether_channel
            }
            data = json.dumps(packet).encode('utf-8')
            self._ether_socket.sendto(data, (multicast_group, self.ETHER_PORT))
            print(f"[Òtúrá] {self.name} broadcast to Ether: {value}")
            return True
        except Exception as e:
            print(f"[Òtúrá] Ether send error: {e}")
            return False
    
    def ether_gba(self, timeout: float = 5.0) -> Optional[Any]:
        """Receive a message from the Ether network."""
        if not self._listening:
            print(f"[Òtúrá] Error: {self.name} not connected to Ether")
            return None
        
        print(f"[Òtúrá] {self.name} waiting for Ether message...")
        
        import time
        start = time.time()
        
        while time.time() - start < timeout:
            if self._ether_messages:
                message = self._ether_messages.pop(0)
                value = message.get("val")
                print(f"[Òtúrá] {self.name} received from {message['from']}: {value}")
                return value
            time.sleep(0.05)
        
        print(f"[Òtúrá] {self.name} timeout waiting for Ether message")
        return None
    
    def ether_pa(self):
        """Leave the Ether network."""
        was_connected = self._ether_recv_socket or self._ether_socket
        self._ether_running = False
        
        if self._ether_recv_socket:
            self._ether_recv_socket.close()
            self._ether_recv_socket = None
        
        if self._ether_socket:
            self._ether_socket.close()
            self._ether_socket = None
        
        self._ether_messages.clear()
        self._listening = False
        
        if was_connected:
            print(f"[Òtúrá] {self.name} left Ether network")
    
    def on_ether_message(self, callback: Callable):
        """Register a callback for when Ether messages arrive."""
        self._on_message = callback

    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================

    def gbo(self, buffer_size: int = 4096) -> str:
        """gbọ́() - Listen/Receive (Alias for gba)."""
        return self.gba(buffer_size)

    def so_po(self, host: str, port: int) -> bool:
        """so_pọ̀() - Connect (Alias for so)."""
        return self.so(host, port)
    
    # =========================================================================
    # VM-STYLE OPERATIONS
    # =========================================================================
    
    def vm_bind(self, channel: int):
        """VM-style bind to Ether channel."""
        self.ether_de(channel)
    
    def vm_send(self, value: int):
        """VM-style send value to Ether."""
        self.ether_ran(value)
    
    def vm_recv(self, timeout: float = 5.0) -> Optional[int]:
        """VM-style receive from Ether."""
        return self.ether_gba(timeout)
    
    # =========================================================================
    # CLEANUP
    # =========================================================================
    
    def __del__(self):
        """Cleanup on destruction."""
        self.pa()
        self.ether_pa()
