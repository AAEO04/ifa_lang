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
        if (host === 'github.io' || host.endsWith('.github.io')) {
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
            // Check if /docs/ exists in the path (e.g. file:///.../docs/...)
            const docsPos = path.lastIndexOf('/docs/');
            if (docsPos !== -1) {
                const htmlPath = path.substring(docsPos + 6);
                const depth = htmlPath.split('/').filter(Boolean).length - 1;
                return '../'.repeat(Math.max(0, depth)) || './';
            }
            // If not found, and we are serving via HTTP/HTTPS, the depth is the number of path segments
            // because the root of the server is the docs directory.
            if (window.location.protocol.startsWith('http')) {
                const depth = segments.length;
                return '../'.repeat(depth) || './';
            }
            // Fallback for file:// opened without 'docs' in the path
            const htmlPath = path.substring(Math.max(0, path.lastIndexOf('/') + 1));
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
                { href: 'getting-started/installer.html', label: ' Installer Architecture' },
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
                { href: 'language/ajose.html', label: '🔄 Àjọṣe (Reactivity)' },
                { href: 'language/ebo.html', label: '🛡️ Ebo (Resources)' },
                { href: 'language/ewo.html', label: '🚫 Ewo (Assertions)' },
                { href: 'language/native-methods.html', label: '📊 Native Methods' }
            ]
        },
        {
            label: '📚 API',
            items: [
                { href: 'api/api.html', label: '📖 API Reference' }
            ]
        },
        {
            label: '📘 Reference',
            items: [
                { href: 'reference/index.html', label: '📋 Reference Home' },
                { href: 'reference/types.html', label: '🏗️ Type System' },
                { href: 'reference/grammar.html', label: '📐 Grammar' },
                { href: 'reference/reserved-words.html', label: '🔤 Reserved Words' },
                { href: 'reference/memory.html', label: '💾 Memory Management' },
                { href: 'reference/errors.html', label: '🚨 Errors' },
                { href: 'reference/comparison.html', label: '🆚 vs Others' },
                { href: 'reference/migrating-from-python.html', label: '🐍 Python Migration' },
                { href: 'reference/migrating-from-javascript.html', label: '🟨 JS Migration' }
            ]
        },
        {
            label: '💡 Examples',
            items: [
                { href: 'examples/examples.html', label: '📚 Examples Gallery' },
                { href: 'examples/playground.html', label: '🎮 Playground' },
                { href: 'examples/use-cases/index.html', label: '🔧 Use Cases' },
                { href: 'examples/showcase-life.html', label: '🧬 Life Simulation' }
            ]
        },
        {
            label: '🚀 Deployment',
            items: [
                { href: 'deployment/deployment.html', label: '📦 Deployment Guide' },
                { href: 'deployment/embedded.html', label: '⚡ Embedded' },
                { href: 'deployment/oja.html', label: '📦 Oja Manager' },
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
            label: '🎓 Tutorials',
            items: [
                { href: 'tutorials/index.html', label: '📚 All Tutorials' },
                { href: 'tutorials/tour/index.html', label: '🚶 Language Tour' },
                { href: 'tutorials/advanced/index.html', label: '🔬 Advanced' },
                { href: 'tutorials/testing.html', label: '🧪 Testing' },
                { href: 'tutorials/debugging.html', label: '🐛 Debugging' },
                { href: 'tutorials/performance.html', label: '⚡ Performance' }
            ]
        },
        {
            label: '🧩 Domains',
            items: [
                { href: 'domains/index.html', label: '🗺️ All 16 Domains' },
                { href: 'domains/ogbe.html', label: ' Ogbe (System)' },
                { href: 'domains/oyeku.html', label: ' Òyèkú (Random)' },
                { href: 'domains/iwori.html', label: ' Ìwòrì (Time)' },
                { href: 'domains/odi.html', label: ' Òdí (Errors)' },
                { href: 'domains/irosu.html', label: ' Ìròsú (Console)' },
                { href: 'domains/owonrin.html', label: ' Ọ̀wọ́nrín (Collections)' },
                { href: 'domains/obara.html', label: ' Ọ̀bàrà (Math)' },
                { href: 'domains/okanran.html', label: ' Ọ̀kànràn (Debug)' },
                { href: 'domains/ogunda.html', label: ' Ògúndá (Process)' },
                { href: 'domains/osa.html', label: ' Ọ̀sá (Concurrency)' },
                { href: 'domains/ika.html', label: ' Ìká (Strings)' },
                { href: 'domains/oturupon.html', label: ' Òtúúrúpọ̀n (Math)' },
                { href: 'domains/otura.html', label: ' Òtúrá (Networking)' },
                { href: 'domains/irete.html', label: ' Ìrẹtẹ̀ (Crypto)' },
                { href: 'domains/ose.html', label: ' Ọ̀ṣẹ́ (Graphics)' },
                { href: 'domains/ofun.html', label: ' Òfún (Capabilities)' }
            ]
        },
        {
            label: '🔬 Infrastructure',
            items: [
                { href: 'infrastructure/index.html', label: '📋 Overview' },
                { href: 'infrastructure/infra/index.html', label: '🖥️ Infra Modules' },
                { href: 'infrastructure/internals/index.html', label: '⚙️ Internals' }
            ]
        },
        {
            label: '🌍 Community',
            items: [
                { href: 'community/community.html', label: '👥 Community Hub' },
                { href: 'community/contributing.html', label: '🤝 Contributing' },
                { href: 'community/babalawo.html', label: '🧙‍♂️ Babalawo' },
                { href: 'community/changelog.html', label: '📋 Changelog' },
                { href: 'community/faq.html', label: '❓ FAQ' }
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

    // Build breadcrumbs from path
    function buildBreadcrumbs() {
        const list = document.querySelector('.breadcrumb-list');
        if (!list) return;

        const path = window.location.pathname;
        const segments = path.replace(/\.html$/, '').split('/').filter(Boolean);

        // Find the docs root segment index
        const docsIdx = segments.indexOf('docs');
        if (docsIdx === -1) return;

        // Extract path segments after 'docs/' (exclude the filename itself)
        const dirSegments = segments.slice(docsIdx + 1, -1);
        const fileName = segments[segments.length - 1];

        // Build breadcrumb HTML
        const crumbs = [{ label: '🏠 Home', href: ROOT + 'index.html' }];

        // Accumulate path for intermediate crumbs
        let accumulated = '';
        for (const seg of dirSegments) {
            accumulated += seg + '/';
            const label = breadcrumbLabel(seg);
            crumbs.push({ label, href: ROOT + accumulated + 'index.html' });
        }

        // Current page — get title from h1 or document
        const h1 = document.querySelector('h1');
        const pageLabel = h1 ? h1.textContent.trim() : (fileName || 'Page');
        crumbs.push({ label: pageLabel, href: null });

        // Render
        list.innerHTML = crumbs.map((c, i) => {
            const sep = i > 0 ? '<span class="breadcrumb-sep">›</span>' : '';
            if (c.href) {
                return `<li>${sep}<a href="${c.href}">${c.label}</a></li>`;
            }
            return `<li>${sep}<span class="current">${c.label}</span></li>`;
        }).join('');
    }

    function breadcrumbLabel(seg) {
        const map = {
            'language': 'Language Reference',
            'domains': 'Domains',
            'infrastructure': 'Infrastructure',
            'infra': 'Infrastructure',
            'internals': 'Internals',
            'tutorials': 'Tutorials',
            'tour': 'Tour',
            'advanced': 'Advanced',
            'dev': 'Dev Docs',
            'maintainers': 'Maintainers',
            'community': 'Community',
            'examples': 'Examples',
            'use-cases': 'Use Cases',
            'deployment': 'Deployment',
            'reference': 'Reference',
            'tools': 'Tools',
            'getting-started': 'Getting Started',
            'api': 'API'
        };
        return map[seg] || seg.charAt(0).toUpperCase() + seg.slice(1);
    }

    // Insert navigation
    document.addEventListener('DOMContentLoaded', function () {
        const navPlaceholder = document.getElementById('nav-placeholder');
        if (navPlaceholder) {
            navPlaceholder.innerHTML = buildNav();

            // Load search script
            const script = document.createElement('script');
            script.src = ROOT + 'js/search.js';
            document.body.appendChild(script);
            // Load highlight script
            const highlightScript = document.createElement('script');
            highlightScript.src = ROOT + 'js/highlight.js';
            document.body.appendChild(highlightScript);
        }

        buildBreadcrumbs();

        // Load universal language switcher (for all pages with code)
        const langSwitcherStyle = document.createElement('link');
        langSwitcherStyle.rel = 'stylesheet';
        langSwitcherStyle.href = ROOT + 'js/language-switcher.css';
        document.head.appendChild(langSwitcherStyle);

        const langSwitcherScript = document.createElement('script');
        langSwitcherScript.src = ROOT + 'js/language-switcher.js';
        document.head.appendChild(langSwitcherScript);
    });

    // Export for use in other scripts
    window.getBasePath = getBasePath;
    window.IFA_DOCS = {
        ROOT: ROOT,
        getBasePath: getBasePath
    };
})();
