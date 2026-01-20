/**
 * Dynamic Navigation System for Ifá-Lang Docs
 * Solves the relative path problem by calculating paths at runtime
 */

(function () {
    // Calculate path to docs root based on current page location
    function getBasePath() {
        const path = window.location.pathname;
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
                { href: 'reference/comparison.html', label: '🆚 vs Others' }
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
        }
    ];

    // Build navigation HTML
    function buildNav() {
        let html = `
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

          <button class="nav-toggle" onclick="toggleNav()">☰</button>
          <nav>
            <ul class="nav-menu" id="nav-menu">
    `;

        for (const section of navItems) {
            html += `
              <li class="nav-dropdown">
                <a href="#">${section.label}</a>
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
        menu.classList.toggle('nav-open');
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
