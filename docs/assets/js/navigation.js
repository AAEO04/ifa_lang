// Navigation Component JavaScript
class NavigationComponent {
    constructor() {
        this.init();
    }

    init() {
        this.setupMobileToggle();
        this.setupDropdowns();
        this.setupActiveNavigation();
        this.generateBreadcrumbs();
    }

    setupMobileToggle() {
        const toggle = document.querySelector('.nav-toggle');
        const menu = document.querySelector('.nav-menu');
        
        if (toggle && menu) {
            toggle.addEventListener('click', () => {
                menu.classList.toggle('active');
                toggle.textContent = menu.classList.contains('active') ? '✕' : '☰';
            });
        }
    }

    setupDropdowns() {
        const dropdowns = document.querySelectorAll('.nav-dropdown');
        
        dropdowns.forEach(dropdown => {
            const link = dropdown.querySelector('a:first-child');
            
            if (link) {
                link.addEventListener('click', (e) => {
                    if (window.innerWidth <= 768) {
                        e.preventDefault();
                        dropdown.classList.toggle('active');
                    }
                });
            }
        });

        // Close dropdowns when clicking outside
        document.addEventListener('click', (e) => {
            if (!e.target.closest('.nav-dropdown')) {
                dropdowns.forEach(dropdown => {
                    dropdown.classList.remove('active');
                });
            }
        });
    }

    setupActiveNavigation() {
        const currentPath = window.location.pathname;
        const navLinks = document.querySelectorAll('.nav-menu a');
        
        navLinks.forEach(link => {
            const linkPath = new URL(link.href).pathname;
            
            // Check if this link matches the current path
            if (currentPath === linkPath || 
                (linkPath !== '/' && currentPath.startsWith(linkPath))) {
                link.classList.add('active');
            } else {
                link.classList.remove('active');
            }
        });
    }

    generateBreadcrumbs() {
        const breadcrumbContainer = document.querySelector('.breadcrumb-list');
        if (!breadcrumbContainer) return;

        const rootPrefix = this.getRootPrefix();
        const pathParts = this.getPathParts(rootPrefix);
        const breadcrumbs = this.buildBreadcrumbList(pathParts, rootPrefix);
        
        breadcrumbContainer.innerHTML = breadcrumbs;
    }

    /// Extract the relative prefix (e.g. "../../" or "../" or "") from the script tag src
    getRootPrefix() {
        const script = document.querySelector('script[src*="navigation.js"]') || document.querySelector('script[src*="nav.js"]');
        if (script) {
            const src = script.getAttribute('src');
            const match = src.match(/^(\.\.\/)*\.\.\/?|^(\.\/)?/);
            if (match) {
                return match[0];
            }
        }
        return '';
    }

    getPathParts(rootPrefix) {
        const pathSegments = window.location.pathname.split('/').filter(s => s);
        if (pathSegments.length === 0) return [];
        
        const depth = (rootPrefix.match(/\.\.\//g) || []).length;
        const fileSegment = pathSegments[pathSegments.length - 1];
        
        // Take directory segments prior to the filename based on the depth
        const dirSegments = pathSegments.slice(
            Math.max(0, pathSegments.length - 1 - depth),
            pathSegments.length - 1
        );
        
        return [...dirSegments, fileSegment];
    }

    buildBreadcrumbList(parts, rootPrefix) {
        let breadcrumbs = `<li><a href="${rootPrefix}index.html">🏠 Home</a></li>`;
        let accumulatedPath = '';
        
        parts.forEach((part, index) => {
            const isLast = index === parts.length - 1;
            
            if (part.toLowerCase() === 'index.html') {
                return;
            }
            
            accumulatedPath += (accumulatedPath ? '/' : '') + part;
            
            if (isLast) {
                const displayName = this.getDisplayName(part, true);
                breadcrumbs += `<li class="separator">›</li><li class="current">${displayName}</li>`;
            } else {
                const displayName = this.getDisplayName(part, false);
                const relativeLink = `${rootPrefix}${accumulatedPath}/index.html`;
                breadcrumbs += `<li class="separator">›</li><li><a href="${relativeLink}">${displayName}</a></li>`;
            }
        });
        
        return breadcrumbs;
    }

    getDisplayName(part, isLast = false) {
        if (isLast) {
            // Attempt to get the actual page header or document title
            const h1 = document.querySelector('h1');
            if (h1 && h1.textContent.trim()) {
                return h1.textContent.trim();
            }
            const title = document.title;
            if (title) {
                return title.split(' - ')[0].trim();
            }
        }

        const categoryEmojis = {
            'getting-started': '🚀',
            'language': '📖',
            'api': '📚',
            'examples': '💡',
            'deployment': '🚢',
            'tools': '🔧',
            'community': '🌍',
            'infrastructure': '🏗️',
            'tutorials': '🎓',
            'advanced': '🧠',
            'reference': '📋',
            'embedded': '🔌',
            'use-cases': '🔧'
        };

        const emoji = categoryEmojis[part.toLowerCase()] || '';
        const formatted = part
            .split('-')
            .map(word => word.charAt(0).toUpperCase() + word.slice(1))
            .join(' ');

        return emoji ? `${emoji} ${formatted}` : formatted;
    }
}

// Initialize navigation when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new NavigationComponent();
});

// Export for potential use in other scripts
if (typeof module !== 'undefined' && module.exports) {
    module.exports = NavigationComponent;
}
