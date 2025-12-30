# -*- coding: utf-8 -*-
import sys
import json
import logging
from typing import Dict, Any

# Configure logging
logging.basicConfig(filename='ifa_lsp.log', level=logging.DEBUG)

class IfaLanguageServer:
    def __init__(self):
        self.running = True
        self.buffer = ""

    def start(self):
        """Start the LSP server over stdio."""
        logging.info("Ifá LSP Server Started")
        while self.running:
            try:
                self.handle_message()
            except Exception as e:
                logging.error(f"Error: {e}")
                self.running = False

    def handle_message(self):
        """Read header and body, then process."""
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

    def process_request(self, request: Dict[str, Any]):
        """Handle JSON-RPC request."""
        logging.debug(f"Received: {request}")
        
        method = request.get("method")
        msg_id = request.get("id")
        params = request.get("params")
        
        response = {
            "jsonrpc": "2.0",
            "id": msg_id
        }
        
        if method == "initialize":
            response["result"] = {
                "capabilities": {
                    "textDocumentSync": 1, # Full sync for simplicity
                    "completionProvider": {
                        "resolveProvider": False,
                        "triggerCharacters": ["."]
                    },
                    "hoverProvider": True
                }
            }
            self.send_response(response)
            
        elif method == "textDocument/completion":
            # Simple static completion for now
            items = []
            
            # Odù Domains
            domains = [
                "Ogbe", "Oyeku", "Iwori", "Odi", "Irosu", "Owonrin", 
                "Obara", "Okanran", "Ogunda", "Osa", "Ika", 
                "Oturupon", "Otura", "Irete", "Ose", "Ofun"
            ]
            for d in domains:
                items.append({
                    "label": d,
                    "kind": 7, # Class
                    "detail": "Odù Ifá Domain"
                })
                
            # Keywords
            keywords = ["ayanmọ", "iba", "fun", "pada", "inu", "bi", "abi", "kaka_ewu", "dida_ewu", "eewo", "ase_pataki"]
            for k in keywords:
                items.append({
                    "label": k,
                    "kind": 14, # Keyword
                    "detail": "Ifá Keyword"
                })

            # Standard Library Functions (Yoruba & English)
            std_funcs = [
                # Yoruba
                "bi", "gba", "oruko", "env", "ku", "duro", "gbale", "pana", 
                "ago", "royin", "mo", "wo", "fi", "pamo", "ti", "si", "pa", 
                "fo", "so", "pe", "san", "kigbe", "bo", "paaro", "da", 
                "ro", "fikun", "kun", "binu", "je", "gbe", 
                "ge", "mu", "ya", "to", "sa", "ka", "fun", "tu", "wa", "sopo", 
                "din", "pin", "kekere", "aropin", "ran", "de", "gbo", "so_po", 
                "dajo", "dan", "te", "di", "han", "kunle", "botini", "fihan", 
                "ase", "ka_iwe", "ila", "onigun", "iyokoto",
                # English
                "init", "input", "user", "exit", "halt", "gc", "shutdown", 
                "time", "sleep", "report", "know", "look", "write", "read", 
                "open", "close", "save", "lock", "print", "log", "alert", "flush", "error", 
                "rand", "shuffle", "flip", "add", "incr", "mul", "sum", 
                "sub", "div", "mod", "cut", "min", "max", "avg", 
                "create", "push", "pop", "split", "sort", "slice", "len", "format", 
                "compress", "encrypt", "zip", "spawn", "jump", 
                "draw", "show", "render", "button", "display", "sudo", "grant", "docs"
            ]
            for f in std_funcs:
                items.append({
                    "label": f,
                    "kind": 3, # Function
                    "detail": "Standard Library Function"
                })
            
            response["result"] = items
            self.send_response(response)
            
        elif method == "textDocument/hover":
            response["result"] = {
                "contents": "Ifá-Lang Documentation"
            }
            self.send_response(response)
            
        elif method == "shutdown":
            response["result"] = None
            self.send_response(response)
            
        elif method == "exit":
            self.running = False
            
        else:
            # Ignore other notifications
            pass

    def send_response(self, response: Dict[str, Any]):
        """Send JSON-RPC response."""
        body = json.dumps(response)
        message = f"Content-Length: {len(body)}\r\n\r\n{body}"
        sys.stdout.write(message)
        sys.stdout.flush()
        logging.debug(f"Sent: {response}")

def run_server():
    server = IfaLanguageServer()
    server.start()

if __name__ == "__main__":
    run_server()
