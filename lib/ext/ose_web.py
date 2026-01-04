# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    ỌṢẸ́ WEB MIXIN - UI & FRONTEND EXTENSION                   ║
║                    "The Creative Display - Web Edition"                      ║
╚══════════════════════════════════════════════════════════════════════════════╝

Extends OseDomain with web UI capabilities:
- HTML generation
- CSS with Odù-mapped colors
- UI components (buttons, inputs, containers)
- Responsive layouts

Usage:
    from lib.std.ose import OseDomain
    ose = OseDomain()
    html = ose.html_div("content", id="main", odu="ogbe")
    css = ose.css_theme()
"""

from typing import Dict, List, Any, Optional
from dataclasses import dataclass, field


# =============================================================================
# ODU COLOR MAPPING
# =============================================================================

ODU_COLORS = {
    "OGBE": {"primary": "#FFFFFF", "secondary": "#F0F0F0", "accent": "#FFD700"},
    "OYEKU": {"primary": "#000000", "secondary": "#1A1A1A", "accent": "#333333"},
    "IWORI": {"primary": "#FFD700", "secondary": "#FFC107", "accent": "#FF9800"},
    "ODI": {"primary": "#808080", "secondary": "#9E9E9E", "accent": "#616161"},
    "IROSU": {"primary": "#00FF00", "secondary": "#4CAF50", "accent": "#8BC34A"},
    "OWONRIN": {"primary": "#FF00FF", "secondary": "#E91E63", "accent": "#9C27B0"},
    "OBARA": {"primary": "#FF0000", "secondary": "#F44336", "accent": "#E53935"},
    "OKANRAN": {"primary": "#0000FF", "secondary": "#2196F3", "accent": "#1976D2"},
    "OGUNDA": {"primary": "#FFA500", "secondary": "#FF9800", "accent": "#F57C00"},
    "OSA": {"primary": "#00FFFF", "secondary": "#00BCD4", "accent": "#0097A7"},
    "IKA": {"primary": "#800080", "secondary": "#9C27B0", "accent": "#7B1FA2"},
    "OTURUPON": {"primary": "#008000", "secondary": "#4CAF50", "accent": "#388E3C"},
    "OTURA": {"primary": "#000080", "secondary": "#3F51B5", "accent": "#303F9F"},
    "IRETE": {"primary": "#800000", "secondary": "#795548", "accent": "#5D4037"},
    "OSE": {"primary": "#008080", "secondary": "#009688", "accent": "#00796B"},
    "OFUN": {"primary": "#C0C0C0", "secondary": "#BDBDBD", "accent": "#757575"},
}


# =============================================================================
# UI COMPONENT DATACLASS
# =============================================================================

@dataclass
class UIComponent:
    """Ifá UI component with Odù state."""
    id: str
    element_type: str
    content: str = ""
    odu_state: str = "OGBE"
    children: List['UIComponent'] = field(default_factory=list)
    attributes: Dict[str, str] = field(default_factory=dict)
    styles: Dict[str, str] = field(default_factory=dict)


# =============================================================================
# OSE WEB MIXIN
# =============================================================================

class OseWebMixin:
    """
    Mixin for OseDomain providing web UI capabilities.
    Adds HTML generation, CSS theming, and UI components.
    """
    
    def __init_web__(self):
        """Initialize web capabilities."""
        self._components: Dict[str, UIComponent] = {}
        self._current_odu = "OGBE"
    
    # =========================================================================
    # ODU THEMING
    # =========================================================================
    
    def odu_awo(self, odu: str = "OGBE") -> Dict[str, str]:
        """Get Odù color palette (àwọ̀ = color)."""
        return ODU_COLORS.get(odu.upper(), ODU_COLORS["OGBE"])
    
    def yi_odu(self, odu: str):
        """Set current Odù theme (yí = change)."""
        self._current_odu = odu.upper()
    
    # =========================================================================
    # HTML GENERATION
    # =========================================================================
    
    def html_nkan(self, tag: str, content: str = "", **attrs) -> str:
        """Generate HTML element (nkan = element/thing)."""
        odu = attrs.pop('odu', self._current_odu)
        colors = self.odu_awo(odu)
        
        style = attrs.pop('style', '')
        if not style:
            style = f"color: {colors['primary']}; background: {colors['secondary']};"
        
        attr_str = ' '.join(f'{k}="{v}"' for k, v in attrs.items())
        if attr_str:
            attr_str = ' ' + attr_str
        
        return f'<{tag}{attr_str} style="{style}">{content}</{tag}>'
    
    def html_div(self, content: str = "", **attrs) -> str:
        """Generate div element."""
        return self.html_nkan('div', content, **attrs)
    
    def html_p(self, content: str = "", **attrs) -> str:
        """Generate paragraph element."""
        return self.html_nkan('p', content, **attrs)
    
    def html_h1(self, content: str = "", **attrs) -> str:
        """Generate h1 element."""
        return self.html_nkan('h1', content, **attrs)
    
    def html_botini(self, label: str, **attrs) -> str:
        """Generate button (bọtini = button)."""
        odu = attrs.pop('odu', self._current_odu)
        colors = self.odu_awo(odu)
        
        style = f"""
            background: {colors['accent']};
            color: {colors['primary']};
            border: none;
            padding: 10px 20px;
            border-radius: 5px;
            cursor: pointer;
        """.replace('\n', ' ').strip()
        
        attrs['style'] = style
        return self.html_nkan('button', label, **attrs)
    
    def html_igbanwo(self, placeholder: str = "", **attrs) -> str:
        """Generate input (ìgbànwọ́ = input/receiver)."""
        odu = attrs.pop('odu', self._current_odu)
        colors = self.odu_awo(odu)
        
        attrs['placeholder'] = placeholder
        attrs['style'] = f"border: 2px solid {colors['accent']}; padding: 8px;"
        
        attr_str = ' '.join(f'{k}="{v}"' for k, v in attrs.items())
        return f'<input {attr_str} />'
    
    # =========================================================================
    # CSS GENERATION
    # =========================================================================
    
    def css_odu(self, odu: str = None) -> str:
        """Generate CSS variables for Odù theme."""
        odu = (odu or self._current_odu).upper()
        colors = self.odu_awo(odu)
        
        return f"""
:root {{
    --ifa-primary: {colors['primary']};
    --ifa-secondary: {colors['secondary']};
    --ifa-accent: {colors['accent']};
    --ifa-odu: "{odu}";
}}
""".strip()
    
    def css_akojo(self) -> str:
        """Generate full CSS theme collection (akojọ = collection)."""
        styles = [":root { font-family: 'Segoe UI', sans-serif; }"]
        
        for odu, colors in ODU_COLORS.items():
            styles.append(f"""
.odu-{odu.lower()} {{
    color: {colors['primary']};
    background: {colors['secondary']};
    border-color: {colors['accent']};
}}
""")
        
        return '\n'.join(styles)
    
    # =========================================================================
    # COMPONENT MANAGEMENT
    # =========================================================================
    
    def da_nkan(self, id: str, element_type: str = "div", 
                content: str = "", odu: str = None) -> UIComponent:
        """Create UI component (dá = create, nkan = thing)."""
        odu = (odu or self._current_odu).upper()
        
        comp = UIComponent(
            id=id,
            element_type=element_type,
            content=content,
            odu_state=odu,
            styles=self.odu_awo(odu)
        )
        self._components[id] = comp
        return comp
    
    def gba_nkan(self, id: str) -> Optional[UIComponent]:
        """Get component by ID (gba = get/receive)."""
        return self._components.get(id)
    
    def yi_ipo(self, comp: UIComponent, new_odu: str):
        """Transition component to new Odù (yí ipò = change state)."""
        comp.odu_state = new_odu.upper()
        comp.styles = self.odu_awo(new_odu)
    
    def han_nkan(self, comp: UIComponent) -> str:
        """Render component to HTML (hàn = show)."""
        style = '; '.join(f"{k}: {v}" for k, v in comp.styles.items())
        attrs = ' '.join(f'{k}="{v}"' for k, v in comp.attributes.items())
        
        children_html = ''.join(self.han_nkan(c) for c in comp.children)
        content = comp.content + children_html
        
        return f'<{comp.element_type} id="{comp.id}" {attrs} style="{style}">{content}</{comp.element_type}>'
    
    # =========================================================================
    # PAGE GENERATION
    # =========================================================================
    
    def html_oju_opo(self, title: str = "Ifá App", body: str = "", odu: str = None) -> str:
        """Generate full HTML page (ojú-ìwé = page)."""
        odu = (odu or self._current_odu).upper()
        colors = self.odu_awo(odu)
        
        return f"""<!DOCTYPE html>
<html lang="yo">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        {self.css_odu(odu)}
        body {{
            font-family: 'Segoe UI', sans-serif;
            background: var(--ifa-secondary);
            color: var(--ifa-primary);
            margin: 0;
            padding: 20px;
        }}
    </style>
</head>
<body>
    {body}
</body>
</html>"""


# =============================================================================
# EXPORT
# =============================================================================

__all__ = ['OseWebMixin', 'UIComponent', 'ODU_COLORS']
