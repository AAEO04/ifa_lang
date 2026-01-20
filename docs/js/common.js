/**
 * Ifá-Lang Documentation Common JavaScript
 * Version: 1.2.2
 */

// Initialize language switcher universally
function initLanguageSwitcher() {
    // Only initialize if language switcher is available
    if (typeof enhanceAllCodeExamples === 'function') {
        enhanceAllCodeExamples();
        console.log('Language switcher initialized for this page');
    }
}

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
        runBtn.style.color = 'var(--success)';
        runBtn.style.border = '1px solid var(--success)';

        runBtn.onclick = () => {
            const code = pre.querySelector('code') || pre;
            try {
                // Unicode-safe Base64 encoding
                const encoded = btoa(unescape(encodeURIComponent(code.textContent)));

                // Determine path to playground
                const isDeep = window.location.pathname.includes('/domains/') || window.location.pathname.includes('/stacks/');
                const playgroundPath = isDeep ? '../playground.html' : 'playground.html';

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

        // --- Language Toggle Button ---
        const langBtn = document.createElement('button');
        langBtn.className = 'action-btn lang-btn';
        langBtn.textContent = '🌍 EN';
        langBtn.title = 'Toggle Yoruba/English';
        langBtn.style.background = 'rgba(255,215,0,0.2)';
        langBtn.style.color = 'var(--gold)';
        langBtn.style.border = '1px solid var(--gold)';
        langBtn.onclick = () => {
            const newLang = currentLang === 'yoruba' ? 'english' : 'yoruba';
            translateCode(newLang);
            // Update all toggle buttons
            document.querySelectorAll('.lang-btn').forEach(btn => {
                btn.textContent = newLang === 'yoruba' ? '🌍 EN' : '🌍 YO';
            });
        };

        btnContainer.appendChild(langBtn);
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
    const version = '1.2.2';
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

// ═══════════════════════════════════════════════════════════════════════════
// BILINGUAL CODE TRANSLATION SYSTEM
// Integration with language-switcher.js for comprehensive word mappings
// ═══════════════════════════════════════════════════════════════════════════

// Import wordMappings and functions from language-switcher.js if not already loaded
// The language-switcher.js file provides:
// - wordMappings: comprehensive Yoruba/English word mappings (200+ words)
// - reverseLookup: efficient reverse lookup table
// - initializeSwitchableWords(): wraps keywords in clickable spans
// - setLang(lang): switches all words to target language
// - switchWord(element): toggle individual word on click

// Current language state (shared with language-switcher.js)
let currentLang = 'yoruba';

// Check if language-switcher.js is loaded
function isLanguageSwitcherLoaded() {
    return typeof wordMappings !== 'undefined' && typeof reverseLookup !== 'undefined';
}

// Fallback simple keyword map if language-switcher.js not loaded
const FALLBACK_MAP = {
    'ayanmo': 'let', 'ti': 'if', 'bibẹkọ': 'else', 'fun': 'for',
    'nigba': 'while', 'padapọ': 'return', 'ise': 'fn', 'otito': 'true',
    'iro': 'false', 'Irosu': 'Fmt', 'Ogunda': 'List', 'Ika': 'String',
    '.fo(': '.println(', '.so(': '.print('
};

/**
 * Toggle language using language-switcher.js if available, otherwise use fallback
 */
function translateCode(targetLang) {
    if (targetLang === currentLang) return;

    if (isLanguageSwitcherLoaded()) {
        // Use the comprehensive language-switcher.js system
        if (typeof setLang === 'function') {
            setLang(targetLang);
        }
    } else {
        // Fallback: simple regex replacement
        document.querySelectorAll('pre code, pre').forEach(codeEl => {
            if (!codeEl._originalText) {
                codeEl._originalText = codeEl.textContent;
            }

            let text = codeEl._originalText;
            if (targetLang === 'english') {
                for (const [yoruba, english] of Object.entries(FALLBACK_MAP)) {
                    if (yoruba.startsWith('.')) {
                        text = text.split(yoruba).join(english);
                    } else {
                        text = text.replace(new RegExp(`\\b${yoruba}\\b`, 'g'), english);
                    }
                }
            }
            codeEl.textContent = text;
        });
    }

    currentLang = targetLang;
    localStorage.setItem('ifa-lang-pref', targetLang);
    updateLangButtons(targetLang);
}

function updateLangButtons(lang) {
    document.querySelectorAll('.lang-btn').forEach(btn => {
        btn.textContent = lang === 'yoruba' ? '🌍 EN' : '🌍 YO';
    });
    document.querySelectorAll('.toggle-yoruba').forEach(btn => {
        btn.classList.toggle('active', lang === 'yoruba');
    });
    document.querySelectorAll('.toggle-english').forEach(btn => {
        btn.classList.toggle('active', lang === 'english');
    });
}

// Initialize on load
document.addEventListener('DOMContentLoaded', () => {
    initCodeActions();
    addVersionFooter();
    initSearch();
    addAnchorLinks();

    // Load preference
    const pref = localStorage.getItem('ifa-lang-pref') || 'yoruba';
    currentLang = 'yoruba';

    // Initialize language-switcher.js if loaded
    if (isLanguageSwitcherLoaded() && typeof enhanceAllCodeExamples === 'function') {
        enhanceAllCodeExamples();
    }

    // Apply saved preference
    if (pref === 'english') {
        setTimeout(() => translateCode('english'), 100);
    }
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
            anchor.style.color = 'var(--text-dim)';
            anchor.style.fontSize = '0.8em';

            heading.appendChild(anchor);

            heading.addEventListener('mouseenter', () => anchor.style.opacity = '1');
            heading.addEventListener('mouseleave', () => anchor.style.opacity = '0');
        }
    });
}
