/**
 * Dynamic Navigation System for Ifá-Lang Docs
 * Solves the relative path problem by calculating paths at runtime
 */

(function () {
    // Calculate path to docs root based on current page location
    function getBasePath() {
        const path = window.location.pathname;
        const host = window.location.hostname;

        // GitHub Pages: aaeo04.github.io/ifa_lang/...
        // The /ifa_lang/ folder IS the docs root
        if (host.includes('github.io')) {
            // Count depth from the repo root (first segment after host)
            const segments = path.split('/').filter(s => s && !s.endsWith('.html'));
            // segments[0] is 'ifa_lang', so depth = segments.length - 1
            const depth = Math.max(0, segments.length - 1);
            return '../'.repeat(depth) || './';
        }

        // Local development: file:///path/to/ifa_lang/docs/...
        const segments = path.split('/').filter(s => s && !s.endsWith('.html'));

        // Find 'docs' in the path and count depth after it
        const docsIndex = segments.findIndex(s => s === 'docs');
        if (docsIndex === -1) {
            // If opened as file://, count from the HTML file
            const htmlPath = path.substring(path.lastIndexOf('/docs/') + 6);
            const depth = htmlPath.split('/').length - 1;
            return '../'.repeat(depth) || './';
        }

        const depth = segments.length - docsIndex - 1;
        return '../'.repeat(depth) || './';
    }

    const ROOT = getBasePath();

    // Navigation structure
    const navItems = [
        {
            label: '🚀 Getting Started',
            items: [
                { href: 'getting-started/quickstart.html', label: ' Quick Start' },
                { href: 'getting-started/install.html', label: ' Installation' },
                { href: 'getting-started/hello-world.html', label: ' Hello World' }
            ]
        },
        {
            label: '📖 Language',
            items: [
                { href: 'language/syntax.html', label: '📝 Syntax' },
                { href: 'language/types-crate.html', label: '🏗️ Types' },
                { href: 'language/macros.html', label: '⚙️ Macros' },
                { href: 'language/philosophy.html', label: '🔮 Philosophy' },
                { href: 'reference/comparison.html', label: '🆚 vs Others' },
                { href: 'reference/migrating-from-python.html', label: '🐍 Python Migration' },
                { href: 'reference/migrating-from-javascript.html', label: '🟨 JS Migration' }
            ]
        },
        {
            label: '📚 API',
            items: [
                { href: 'api/api.html', label: '📖 API Reference' }
            ]
        },
        {
            label: '💡 Examples',
            items: [
                { href: 'examples/examples.html', label: '📚 Examples Gallery' },
                { href: 'examples/playground.html', label: '🎮 Playground' },
                { href: 'examples/use-cases/index.html', label: '🔧 Use Cases' }
            ]
        },
        {
            label: '🚀 Deployment',
            items: [
                { href: 'deployment/deployment.html', label: '📦 Deployment Guide' },
                { href: 'deployment/oja-publishing.html', label: '📤 Oja Publishing' }
            ]
        },
        {
            label: '🔧 Tools',
            items: [
                { href: 'tools/cli.html', label: '⌨️ CLI' },
                { href: 'tools/ide-integration.html', label: '🎨 IDE Integration' },
                { href: 'tools/sandbox.html', label: '🧪 Sandbox' }
            ]
        },
        {
            label: '🌍 Community',
            items: [
                { href: 'community/community.html', label: '👥 Community Hub' },
                { href: 'community/contributing.html', label: '🤝 Contributing' },
                { href: 'community/babalawo.html', label: '🧙‍♂️ Babalawo' }
            ]
        },
        {
            label: '🎓 Tutorials',
            items: [
                { href: 'tutorials/index.html', label: '📚 All Tutorials' },
                { href: 'tutorials/tour/index.html', label: '🚶 Language Tour' }
            ]
        },
        {
            label: '🔧 Development',
            items: [
                { href: 'dev/index.html', label: '🗺️ Dev Docs' },
                { href: 'dev/crate-map.html', label: '📦 Crate Map' },
                { href: 'dev/vm.html', label: '⚡ VM Internals' },
                { href: 'dev/compiler.html', label: '📦 Compiler' },
                { href: 'dev/value-system.html', label: '🔢 Value System' },
                { href: 'dev/adding-a-domain.html', label: '📦 External Packages' },
                { href: 'dev/testing.html', label: '🧪 Testing' }
            ]
        },
        {
            label: '🔐 Maintainers',
            items: [
                { href: 'maintainers/index.html', label: '📋 Maintainer Docs' },
                { href: 'maintainers/release-process.html', label: '📤 Release Process' },
                { href: 'maintainers/ci-cd.html', label: '⚙️ CI/CD' }
            ]
        }
    ];

    // Build navigation HTML
    function buildNav() {
        let html = `
      <a href="#main-content" class="skip-link">Skip to main content</a>
      <header class="nav-header">
        <div class="nav-container">
          <a href="${ROOT}index.html" class="nav-logo">
            <span>🔮</span>
            <span>Ifá-Lang</span>
          </a>
          
          <div class="nav-search">
            <input type="text" id="doc-search" placeholder="Search docs..." aria-label="Search documentation">
            <div id="search-results" class="search-results"></div>
          </div>

          <button class="nav-toggle" onclick="toggleNav()" aria-label="Toggle navigation menu" aria-expanded="false">☰</button>
          <nav role="navigation" aria-label="Main navigation">
            <ul class="nav-menu" id="nav-menu">
    `;

        for (const section of navItems) {
            html += `
              <li class="nav-dropdown">
                <a href="#" onclick="toggleDropdown(event, this)">${section.label}</a>
                <div class="nav-dropdown-content">
      `;
            for (const item of section.items) {
                html += `          <a href="${ROOT}${item.href}">${item.label}</a>\n`;
            }
            html += `        </div>
              </li>
      `;
        }

        html += `
            </ul>
          </nav>
        </div>
      </header>
    `;

        return html;
    }

    // Toggle mobile nav
    window.toggleNav = function () {
        const menu = document.getElementById('nav-menu');
        const btn = document.querySelector('.nav-toggle');
        const isOpen = menu.classList.toggle('nav-open');

        // Update ARIA state
        if (btn) {
            btn.setAttribute('aria-expanded', isOpen);
        }

        // Close all dropdowns when closing menu
        if (!isOpen) {
            document.querySelectorAll('.nav-dropdown').forEach(d => d.classList.remove('dropdown-open'));
        }
    };

    // Toggle mobile dropdown
    window.toggleDropdown = function (event, element) {
        // Only use click behavior on mobile
        if (window.innerWidth > 768) return;

        event.preventDefault();
        event.stopPropagation();

        const dropdown = element.closest('.nav-dropdown');
        const isOpen = dropdown.classList.contains('dropdown-open');

        // Close all other dropdowns
        document.querySelectorAll('.nav-dropdown').forEach(d => d.classList.remove('dropdown-open'));

        // Toggle this one
        if (!isOpen) {
            dropdown.classList.add('dropdown-open');
        }
    };

    // Insert navigation
    document.addEventListener('DOMContentLoaded', function () {
        const navPlaceholder = document.getElementById('nav-placeholder');
        if (navPlaceholder) {
            navPlaceholder.innerHTML = buildNav();

            // Load search script
            const script = document.createElement('script');
            script.src = ROOT + 'js/search.js';
            // Load highlight script
            const highlightScript = document.createElement('script');
            highlightScript.src = ROOT + 'js/highlight.js';
            document.body.appendChild(highlightScript);
        }

        // Always load universal language switcher (for all pages with code)
        const langSwitcherStyle = document.createElement('link');
        langSwitcherStyle.rel = 'stylesheet';
        langSwitcherStyle.href = ROOT + 'js/language-switcher.css';
        document.head.appendChild(langSwitcherStyle);

        const langSwitcherScript = document.createElement('script');
        langSwitcherScript.src = ROOT + 'js/language-switcher.js';
        langSwitcherScript.onload = function () {
            // Initialize language switcher after loading
            if (typeof enhanceAllCodeExamples === 'function') {
                enhanceAllCodeExamples();
            }
        };
        document.head.appendChild(langSwitcherScript);
    });

    // Export for use in other scripts
    window.IFA_DOCS = {
        ROOT: ROOT,
        getBasePath: getBasePath
    };
})();
