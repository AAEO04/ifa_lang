# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           USER COMPOUND TEMPLATE - CREATE YOUR OWN ODÙ                       ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Copy this template to your project's odu/ folder to create custom compounds.║
║                                                                              ║
║  NAMING RULES:                                                               ║
║    1. Filename must be: parent_child.py (e.g., otura_osa.py)                ║
║    2. Class name must be: ParentChild (e.g., OturaOsa)                      ║
║    3. Both parent and child must be valid 16 Principal Odù names            ║
║                                                                              ║
║  VALID PARENTS/CHILDREN:                                                     ║
║    ogbe, oyeku, iwori, odi, irosu, owonrin, obara, okanran,                 ║
║    ogunda, osa, ika, oturupon, otura, irete, ose, ofun                       ║
╚══════════════════════════════════════════════════════════════════════════════╝

EXAMPLE: Create a WebSocket module as Òtúrá_Ọ̀sá (Network + Speed)

1. Create file: your_project/odu/otura_osa.py
2. Copy this template
3. Implement your methods
4. It will be auto-loaded when you run `ifa run`

"""


class OturaOsa:
    """
    Òtúrá_Ọ̀sá: Fast Network Communication
    
    Parent: Òtúrá (Network/Communication)
    Child:  Ọ̀sá (Wind/Speed)
    Meaning: "Swift Communication" = WebSockets / Streaming
    
    Opcode: 0xC9 (Parent: 0xC, Child: 0x9)
    """
    
    # Class-level state (shared across all calls)
    _connections = {}
    
    @classmethod
    def sopọ(cls, url: str, name: str = "default") -> bool:
        """
        sọpọ̀ = Connect (Open WebSocket connection)
        
        Args:
            url: WebSocket URL (ws:// or wss://)
            name: Connection name for reference
            
        Returns:
            True if connection opened
        """
        # TODO: Implement actual WebSocket connection
        cls._connections[name] = {"url": url, "status": "connected"}
        print(f"🔌 [Òtúrá_Ọ̀sá] Connected to: {url}")
        return True
    
    @classmethod
    def ran(cls, message: str, name: str = "default") -> bool:
        """
        rán = Send (Send message through WebSocket)
        
        Args:
            message: Message to send
            name: Connection name
            
        Returns:
            True if sent successfully
        """
        if name not in cls._connections:
            print(f"❌ [Òtúrá_Ọ̀sá] No connection: {name}")
            return False
        
        # TODO: Implement actual sending
        print(f"📤 [Òtúrá_Ọ̀sá] Sent: {message}")
        return True
    
    @classmethod
    def gba(cls, name: str = "default") -> str:
        """
        gbà = Receive (Receive message from WebSocket)
        
        Args:
            name: Connection name
            
        Returns:
            Received message or empty string
        """
        if name not in cls._connections:
            return ""
        
        # TODO: Implement actual receiving
        return ""
    
    @classmethod
    def ya(cls, name: str = "default") -> bool:
        """
        yà = Disconnect (Close WebSocket connection)
        
        Args:
            name: Connection name
            
        Returns:
            True if disconnected
        """
        if name in cls._connections:
            del cls._connections[name]
            print(f"🔌 [Òtúrá_Ọ̀sá] Disconnected: {name}")
            return True
        return False


# =============================================================================
# MODULE-LEVEL FUNCTIONS (for direct Ifá-Lang access)
# =============================================================================
# These allow: Òtúrá_Ọ̀sá.sọpọ̀("ws://example.com");

def sopo(url: str, name: str = "default") -> bool:
    return OturaOsa.sopọ(url, name)

def ran(message: str, name: str = "default") -> bool:
    return OturaOsa.ran(message, name)

def gba(name: str = "default") -> str:
    return OturaOsa.gba(name)

def ya(name: str = "default") -> bool:
    return OturaOsa.ya(name)
