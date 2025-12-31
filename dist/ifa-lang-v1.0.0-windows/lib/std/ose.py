# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ỌṢẸ́ - THE BEAUTIFIER (1010)                                        ║
║                    Graphics & Display                                        ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

from .base import OduModule


class OseDomain(OduModule):
    """The Beautifier - Graphics and display."""
    
    def __init__(self):
        super().__init__("Ọ̀ṣẹ́", "1010", "The Beautifier - Graphics")
        
        # Configurable canvas
        self.width = 30
        self.height = 15
        self.buffer = [[' ' for _ in range(self.width)] for _ in range(self.height)]
        self.cursor_x = 0
        self.cursor_y = 0
        
        # Core Draw
        self._register("nu", self.nu, "Clear canvas")
        self._register("ya", self.ya, "Draw pixel")
        self._register("han", self.han, "Render canvas")
        self._register("tobi", self.tobi, "Resize canvas")
        
        # Shapes
        self._register("ila", self.ila, "Draw line")
        self._register("onigun", self.onigun, "Draw rectangle")
        self._register("onigun_kun", self.onigun_kun, "Filled rectangle")
        self._register("iyokoto", self.iyokoto, "Draw circle")
        
        # Text & Fill
        self._register("ko", self.ko, "Write text on canvas")
        self._register("kun", self.kun, "Flood fill")
        
        # Cursor
        self._register("fi_x", self.fi_x, "Set cursor X")
        self._register("fi_y", self.fi_y, "Set cursor Y")
        self._register("ya_nibi", self.ya_nibi, "Draw at cursor")
        
        # Spec Functions
        self._register("kunle", self.kunle, "Render (Alias for han)")
        self._register("awo", self.awo, "Color (Stub)")
        self._register("botini", self.botini, "Button (Stub)")
        self._register("fihan", self.fihan, "Show (Alias for han)")
        
        # VM opcodes
        self.OPCODES = {
            "G_CLR": "10101111",
            "G_DRAW": "10101100",
            "G_SHOW": "10100000",
            "SET_X": "10101001",
            "SET_Y": "10101010",
        }
    
    # =========================================================================
    # CORE DRAW
    # =========================================================================
    
    def nu(self, fill_char: str = '.'):
        """Clear canvas with specified character."""
        self.buffer = [[fill_char for _ in range(self.width)] for _ in range(self.height)]
    
    def ya(self, x: int, y: int, char: str = '#'):
        """Draw character at position."""
        if 0 <= x < self.width and 0 <= y < self.height:
            self.buffer[y][x] = char[0] if char else '#'
    
    def han(self):
        """Render canvas to console."""
        print("\n[Ọ̀ṣẹ́] Rendering Frame:")
        print("+" + "-" * self.width + "+")
        for row in self.buffer:
            print("|" + "".join(row) + "|")
        print("+" + "-" * self.width + "+")
        
    def tobi(self, width: int, height: int):
        """Resize canvas."""
        self.width = width
        self.height = height
        self.nu('.')

    # =========================================================================
    # SHAPES
    # =========================================================================
    
    def ila(self, x1: int, y1: int, x2: int, y2: int, char: str = '#'):
        """Draw line using Bresenham's algorithm."""
        dx = abs(x2 - x1)
        dy = abs(y2 - y1)
        sx = 1 if x1 < x2 else -1
        sy = 1 if y1 < y2 else -1
        err = dx - dy
        
        while True:
            self.ya(x1, y1, char)
            if x1 == x2 and y1 == y2:
                break
            e2 = 2 * err
            if e2 > -dy:
                err -= dy
                x1 += sx
            if e2 < dx:
                err += dx
                y1 += sy
    
    def onigun(self, x: int, y: int, w: int, h: int, char: str = '#'):
        """Draw rectangle outline."""
        for i in range(x, x + w):
            self.ya(i, y, char)
            self.ya(i, y + h - 1, char)
        for j in range(y, y + h):
            self.ya(x, j, char)
            self.ya(x + w - 1, j, char)
            
    def onigun_kun(self, x: int, y: int, w: int, h: int, char: str = '#'):
        """Draw filled rectangle."""
        for j in range(y, y + h):
            for i in range(x, x + w):
                self.ya(i, j, char)
                
    def iyokoto(self, xc: int, yc: int, r: int, char: str = 'O'):
        """Draw circle using midpoint algorithm."""
        x = 0
        y = r
        d = 3 - 2 * r
        self._circle_points(xc, yc, x, y, char)
        while y >= x:
            x += 1
            if d > 0:
                y -= 1
                d = d + 4 * (x - y) + 10
            else:
                d = d + 4 * x + 6
            self._circle_points(xc, yc, x, y, char)
            
    def _circle_points(self, xc, yc, x, y, char):
        self.ya(xc+x, yc+y, char); self.ya(xc-x, yc+y, char)
        self.ya(xc+x, yc-y, char); self.ya(xc-x, yc-y, char)
        self.ya(xc+y, yc+x, char); self.ya(xc-y, yc+x, char)
        self.ya(xc+y, yc-x, char); self.ya(xc-y, yc-x, char)

    # =========================================================================
    # TEXT & FILL
    # =========================================================================

    def ko(self, x: int, y: int, text: str):
        """Write text at position."""
        for i, char in enumerate(text):
            self.ya(x + i, y, char)
            
    def kun(self, x: int, y: int, char: str = '#'):
        """Flood fill area (basic implementation)."""
        if not (0 <= x < self.width and 0 <= y < self.height):
            return
            
        target_char = self.buffer[y][x]
        if target_char == char:
            return
            
        stack = [(x, y)]
        while stack:
            cx, cy = stack.pop()
            if not (0 <= cx < self.width and 0 <= cy < self.height):
                continue
            if self.buffer[cy][cx] != target_char:
                continue
                
            self.buffer[cy][cx] = char
            
            stack.append((cx+1, cy))
            stack.append((cx-1, cy))
            stack.append((cx, cy+1))
            stack.append((cx, cy-1))

    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================

    def kunle(self):
        """kúnlẹ̀() - Render (Alias for han - "Kneel/Greet")."""
        self.han()

    def awo(self, color_code: str):
        """àwọ̀() - Color (Stub). In console grid, maybe change fill char?"""
        pass

    def botini(self, label: str, x: int, y: int):
        """bọtini() - Button (Stub/Simulated). Draws box with text."""
        w = len(label) + 2
        self.onigun(x, y, w, 3)
        self.ko(x+1, y+1, label)

    def fihan(self):
        """fihàn() - Show (Alias for han)."""
        self.han()

    # =========================================================================
    # CURSOR & VM
    # =========================================================================
    
    def fi_x(self, x: int): self.cursor_x = x
    def fi_y(self, y: int): self.cursor_y = y
    
    def ya_nibi(self, char: str = '#'):
        """Draw at current cursor."""
        self.ya(self.cursor_x, self.cursor_y, char)
    
    # VM-style methods
    def vm_clear(self): self.nu('.')
    def vm_draw(self, c: int = 0): self.ya_nibi('#' if c == 0 else chr(c))
    def vm_show(self): self.han()
    def vm_set_x(self, x: int): self.fi_x(x)
    def vm_set_y(self, y: int): self.fi_y(y)
