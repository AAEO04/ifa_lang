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

        const pathParts = this.getPathParts();
        const breadcrumbs = this.buildBreadcrumbList(pathParts);
        
        breadcrumbContainer.innerHTML = breadcrumbs;
    }

    getPathParts() {
        const path = window.location.pathname;
        // Remove leading/trailing slashes and split
        const parts = path.replace(/^\/|\/$/g, '').split('/');
        
        // Filter out empty parts and handle docs root
        return parts.filter(part => part && part !== 'docs');
    }

    buildBreadcrumbList(parts) {
        let breadcrumbs = '<li><a href="../index.html">🏠 Home</a></li>';
        let currentPath = '';
        
        parts.forEach((part, index) => {
            currentPath += (currentPath ? '/' : '') + part;
            const isLast = index === parts.length - 1;
            
            if (isLast) {
                // Current page - no link
                const displayName = this.getDisplayName(part);
                breadcrumbs += `<li class="separator">›</li><li class="current">${displayName}</li>`;
            } else {
                // Intermediate page - link
                const displayName = this.getDisplayName(part);
                const relativePath = this.getRelativePath(currentPath, parts.length);
                breadcrumbs += `<li class="separator">›</li><li><a href="${relativePath}">${displayName}</a></li>`;
            }
        });
        
        return breadcrumbs;
    }

    getDisplayName(part) {
        const displayNames = {
            'getting-started': '🚀 Getting Started',
            'language': '📖 Language',
            'api': '📚 API',
            'examples': '💡 Examples',
            'deployment': '🚀 Deployment',
            'tools': '🔧 Tools',
            'community': '🌍 Community',
            'infrastructure': '🏗️ Infrastructure',
            'tutorials': '🎓 Tutorials',
            'advanced': '🎓 Advanced',
            'tour': '🚶 Tour',
            'use-cases': '🔧 Use Cases',
            'stacks': '📦 Stacks',
            'domains': '🌐 Domains',
            'reference': '📋 Reference',
            'quickstart': '⚡ Quick Start',
            'install': '🔧 Installation',
            'installer': '🏗️ Installer',
            'hello-world': '👋 Hello World',
            'syntax': '📝 Syntax',
            'types-crate': '🏗️ Types',
            'macros': '⚙️ Macros',
            'philosophy': '🔮 Philosophy',
            'api-complete': '📖 Complete API',
            'examples-gallery': '📚 Examples',
            'showcase-life': '🌍 Life Simulation',
            'playground': '🎮 Playground',
            'deployment-guide': '📦 Deployment',
            'oja-publishing': '📤 Publishing',
            'cli': '⌨️ CLI',
            'ide-integration': '🎨 IDE',
            'sandbox': '🧪 Sandbox',
            'community-hub': '👥 Community',
            'contributing': '🤝 Contributing',
            'changelog': '📋 Changelog',
            'babalawo': '🧙‍♂️ Babalawo',
            'infra': '🔧 Infrastructure',
            'internals': '⚙️ Internals',
            'debugging': '🐛 Debugging',
            'ffi': '🔗 FFI',
            'embedded': '🔌 Embedded'
        };
        
        return displayNames[part] || part.charAt(0).toUpperCase() + part.slice(1).replace(/-/g, ' ');
    }

    getRelativePath(currentPath, totalParts) {
        // Calculate relative path based on current depth
        const depth = totalParts - currentPath.split('/').length;
        let relativePath = '';
        
        for (let i = 0; i < depth; i++) {
            relativePath += '../';
        }
        
        relativePath += currentPath + '/index.html';
        return relativePath;
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
