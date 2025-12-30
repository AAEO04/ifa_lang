# -*- coding: utf-8 -*-
import sys
import json
import logging
import threading
from typing import Dict, Any, Optional

# Configure logging
logging.basicConfig(filename='ifa_dap.log', level=logging.DEBUG)

class DebugAdapter:
    """ 
    VS Code Debug Adapter for Ifá-Lang.
    Translates DAP (JSON) -> Ifá Interpreter Actions.
    """
    def __init__(self):
        self.running = True
        self.sequence = 1
        self.breakpoints = {}
        
    def start(self):
        """Start the DAP server over stdio."""
        logging.info("Ifá DAP Server Started")
        while self.running:
            try:
                self.handle_message()
            except Exception as e:
                logging.error(f"DAP Error: {e}")
                self.running = False

    def handle_message(self):
        """Read header and body, then process."""
        try:
            # 1. Read Content-Length
            content_length = 0
            while True:
                line = sys.stdin.readline()
                if not line:
                    self.running = False
                    return
                
                if line.startswith("Content-Length: "):
                    content_length = int(line.split(":")[1].strip())
                
                if line == "\r\n":
                    break
            
            # 2. Read Body
            if content_length > 0:
                body = sys.stdin.read(content_length)
                request = json.loads(body)
                self.process_request(request)
        except Exception:
            self.running = False

    def send_event(self, event: str, body: Dict[str, Any] = None):
        msg = {
            "type": "event",
            "event": event,
            "seq": self.sequence
        }
        if body:
            msg["body"] = body
        self.send_message(msg)

    def send_response(self, request: Dict[str, Any], body: Dict[str, Any] = None, success: bool = True, message: str = None):
        resp = {
            "type": "response",
            "request_seq": request["seq"],
            "command": request["command"],
            "success": success,
            "seq": self.sequence
        }
        if body:
            resp["body"] = body
        if message:
            resp["message"] = message
        self.send_message(resp)

    def send_message(self, msg: Dict[str, Any]):
        self.sequence += 1
        serialized = json.dumps(msg)
        sys.stdout.write(f"Content-Length: {len(serialized)}\r\n\r\n{serialized}")
        sys.stdout.flush()
        logging.debug(f"Sent: {msg}")

    def process_request(self, request: Dict[str, Any]):
        """Handle DAP request."""
        logging.debug(f"Received: {request}")
        command = request.get("command")
        
        if command == "initialize":
            self.send_response(request, {
                "supportsConfigurationDoneRequest": True,
                "supportsHitConditionalBreakpoints": False,
                "supportsConditionalBreakpoints": False,
                "supportsEvaluateForHovers": True
            })
            self.send_event("initialized")
            
        elif command == "launch":
            # In a real debugger, we would start the interpreter in a separate thread/process here
            # For this prototype, we just acknowledge receipt
            self.send_response(request)
            self.send_event("process", {"name": "ifa-vm"})
            # Pretend we stopped on entry
            self.send_event("stopped", {"reason": "entry", "threadId": 1})
            
        elif command == "setBreakpoints":
            # Store breakpoints
            path = request["arguments"]["source"]["path"]
            lines = [bp["line"] for bp in request["arguments"].get("breakpoints", [])]
            self.breakpoints[path] = lines
            
            # Confirm they are verified
            bps = [{"verified": True, "line": l} for l in lines]
            self.send_response(request, {"breakpoints": bps})
            
        elif command == "configurationDone":
            self.send_response(request)
            
        elif command == "threads":
            self.send_response(request, {
                "threads": [{"id": 1, "name": "Main Thread"}]
            })
            
        elif command == "stackTrace":
            self.send_response(request, {
                "stackFrames": [{
                    "id": 1,
                    "name": "main",
                    "source": {"name": "demo.ifa", "path": "c:\\Users\\allio\\Desktop\\ifa_lang\\examples\\demo.ifa"},
                    "line": 1,
                    "column": 1
                }],
                "totalFrames": 1
            })
            
        elif command == "scopes":
            self.send_response(request, {
                "scopes": [{
                    "name": "Locals",
                    "variablesReference": 1,
                    "expensive": False
                }]
            })
            
        elif command == "variables":
            self.send_response(request, {
                "variables": [
                    {"name": "x", "value": "10", "type": "ayanmo", "variablesReference": 0},
                    {"name": "status", "value": "\"Active\"", "type": "string", "variablesReference": 0}
                ]
            })
            
        elif command == "next" or command == "stepIn":
            # Mock stepping
            self.send_response(request)
            self.send_event("stopped", {"reason": "step", "threadId": 1})
            
        elif command == "continue":
            self.send_response(request)
            self.send_event("terminated")
            
        elif command == "disconnect":
            self.send_response(request)
            self.running = False

def run_dap():
    server = DebugAdapter()
    server.start()

if __name__ == "__main__":
    run_dap()
