/**
 * Ifá-Lang Documentation Common JavaScript
 * Version: 1.4.0
 * Language switching lives in language-switcher.js (single source of truth).
 */

// Add copy and run buttons to all pre/code blocks
function initCodeActions() {
    document.querySelectorAll('pre').forEach(pre => {
        // Skip if already has buttons
        if (pre.querySelector('.action-btn')) return;

        // Create wrapper
        const wrapper = document.createElement('div');
        wrapper.className = 'code-container';
        wrapper.style.position = 'relative';

        // Container for buttons (Header)
        const btnContainer = document.createElement('div');
        btnContainer.className = 'code-action-header';
        btnContainer.style.display = 'flex';
        btnContainer.style.justifyContent = 'flex-end';
        btnContainer.style.gap = '0.5rem';
        btnContainer.style.padding = '0.5rem';
        btnContainer.style.background = 'rgba(0, 0, 0, 0.2)';
        btnContainer.style.borderTopLeftRadius = '8px';
        btnContainer.style.borderTopRightRadius = '8px';
        btnContainer.style.borderBottom = '1px solid rgba(233, 69, 96, 0.2)';

        // Remove absolute positioning from wrapper/buttons logic
        // Wrapper now needs to accommodate the header
        wrapper.style.display = 'flex';
        wrapper.style.flexDirection = 'column';
        wrapper.style.borderRadius = '8px';
        wrapper.style.overflow = 'hidden';
        wrapper.style.margin = '1.5rem 0';
        wrapper.style.border = '1px solid rgba(233, 69, 96, 0.2)';

        // Remove default pre margins and radius since wrapper handles it
        pre.style.margin = '0';
        pre.style.borderTopLeftRadius = '0';
        pre.style.borderTopRightRadius = '0';
        pre.style.border = 'none';

        // --- Copy Button ---
        const copyBtn = document.createElement('button');
        copyBtn.className = 'action-btn copy-btn';
        copyBtn.textContent = 'Copy';
        copyBtn.onclick = async () => {
            const code = pre.querySelector('code') || pre;
            try {
                await navigator.clipboard.writeText(code.textContent);
                copyBtn.textContent = 'Copied!';
                copyBtn.classList.add('copied');
                setTimeout(() => {
                    copyBtn.textContent = 'Copy';
                    copyBtn.classList.remove('copied');
                }, 2000);
            } catch (err) {
                copyBtn.textContent = 'Failed';
            }
        };

        // --- Run Button ---
        const runBtn = document.createElement('button');
        runBtn.className = 'action-btn run-btn';
        runBtn.textContent = '▶ Run';
        runBtn.style.background = 'var(--bg-card)';
        runBtn.style.color = '#4af626';
        runBtn.style.border = '1px solid #4af626';

        runBtn.onclick = () => {
            const code = pre.querySelector('code') || pre;
            try {
                // Unicode-safe Base64 encoding
                const encoded = btoa(unescape(encodeURIComponent(code.textContent)));

                // Use nav.js's dynamic pathing system
                const basePath = window.getBasePath ? window.getBasePath() : 
                                 (window.IFA_DOCS && window.IFA_DOCS.getBasePath ? window.IFA_DOCS.getBasePath() : './');
                const playgroundPath = `${basePath}playground.html`;

                window.open(`${playgroundPath}?code=${encoded}`, '_blank');
            } catch (err) {
                console.error('Failed to encode code:', err);
            }
        };

        // styling for buttons
        [copyBtn, runBtn].forEach(btn => {
            btn.style.padding = '4px 12px';
            btn.style.borderRadius = '4px';
            btn.style.cursor = 'pointer';
            btn.style.fontSize = '0.8rem';
            if (btn === copyBtn) {
                btn.style.background = 'rgba(255,255,255,0.1)';
                btn.style.color = 'var(--text)';
                btn.style.border = '1px solid transparent';
            }
        });

        btnContainer.appendChild(runBtn);
        btnContainer.appendChild(copyBtn);

        // Wrap and insert: Wrapper -> [Header, Pre]
        pre.parentNode.insertBefore(wrapper, pre);
        wrapper.appendChild(btnContainer);
        wrapper.appendChild(pre);
    });
}

// Add version footer
function addVersionFooter() {
    const version = '1.3.0';
    const footer = document.querySelector('footer, .doc-footer');
    if (footer && !footer.querySelector('.version')) {
        const versionEl = document.createElement('p');
        versionEl.innerHTML = `Ifá-Lang <span class="version">v${version}</span>`;
        versionEl.style.marginTop = '0.5rem';
        footer.appendChild(versionEl);
    }
}

// Simple fuzzy search for API page
function initSearch() {
    const searchInput = document.getElementById('api-search');
    if (!searchInput) return;

    const cards = document.querySelectorAll('.domain-card');

    searchInput.addEventListener('input', (e) => {
        const query = e.target.value.toLowerCase();
        cards.forEach(card => {
            const text = card.textContent.toLowerCase();
            card.style.display = text.includes(query) ? '' : 'none';
        });
    });
}

// Initialize on load
document.addEventListener('DOMContentLoaded', () => {
    initCodeActions();
    addVersionFooter();
    initSearch();
    addAnchorLinks();
});

// Add anchor links to headings
function addAnchorLinks() {
    document.querySelectorAll('h2, h3').forEach(heading => {
        if (!heading.id) {
            // Generate ID from text
            const id = heading.textContent
                .toLowerCase()
                .replace(/[^a-z0-9]+/g, '-')
                .replace(/(^-|-$)/g, '');
            if (id) heading.id = id;
        }

        if (heading.id) {
            const anchor = document.createElement('a');
            anchor.className = 'anchor-link';
            anchor.href = '#' + heading.id;
            anchor.textContent = '#';
            anchor.style.opacity = '0';
            anchor.style.marginLeft = '0.5rem';
            anchor.style.textDecoration = 'none';
            anchor.style.color = 'var(--dim)';
            anchor.style.fontSize = '0.8em';

            heading.appendChild(anchor);

            heading.addEventListener('mouseenter', () => anchor.style.opacity = '1');
            heading.addEventListener('mouseleave', () => anchor.style.opacity = '0');
        }
    });
}
