/**
 * Simple Client-Side Search for Ifá-Lang Docs
 */
(function () {
    // Basic search index - in a real app this would be generated
    // For now, we'll index key pages dynamically or use a static list
    const searchIndex = [
        { title: "Getting Started", href: "getting-started/quickstart.html", content: "installation guide quick start hello world" },
        { title: "Language Syntax", href: "language/syntax.html", content: "syntax variable types control flow loops functions" },
        { title: "API Reference", href: "api/api.html", content: "api standard library modules domains" },
        { title: "Playground", href: "examples/playground.html", content: "try online browser editor" },
        { title: "CLI Tools", href: "tools/cli.html", content: "command line interface ifa run build" },
        { title: "Odù Philosophy", href: "language/philosophy.html", content: "16 odu domains ifa wisdom" },
        { title: "Tutorials", href: "tutorials/index.html", content: "guides learn tutorial" },
        { title: "Examples", href: "examples/examples.html", content: "code samples snippets" }
    ];

    function initSearch() {
        const searchInput = document.getElementById('doc-search');
        const resultsContainer = document.getElementById('search-results');

        if (!searchInput || !resultsContainer) return;

        searchInput.addEventListener('input', (e) => {
            const query = e.target.value.toLowerCase();

            if (query.length < 2) {
                resultsContainer.style.display = 'none';
                return;
            }

            const results = searchIndex.filter(item =>
                item.title.toLowerCase().includes(query) ||
                item.content.includes(query)
            );

            displayResults(results, resultsContainer);
        });

        // Close search on click outside
        document.addEventListener('click', (e) => {
            if (!e.target.closest('.nav-search')) {
                resultsContainer.style.display = 'none';
            }
        });
    }

    function displayResults(results, container) {
        if (results.length === 0) {
            container.innerHTML = '<div class="no-results">No results found</div>';
        } else {
            const root = window.IFA_DOCS ? window.IFA_DOCS.ROOT : './';
            container.innerHTML = results.map(item => `
                <a href="${root}${item.href}" class="search-result-item">
                    <div class="result-title">${item.title}</div>
                </a>
            `).join('');
        }
        container.style.display = 'block';
    }

    // Wait for nav to be injected
    const observer = new MutationObserver((mutations) => {
        if (document.getElementById('doc-search')) {
            initSearch();
            observer.disconnect();
        }
    });

    observer.observe(document.body, { childList: true, subtree: true });
})();
