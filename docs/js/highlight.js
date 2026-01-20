/**
 * Lightweight Syntax Highlighter for Ifá-Lang
 * Supports Yoruba and English keywords
 */
(function () {
    const KEYWORDS_YORUBA = [
        'ayanmo', 'ti', 'bibẹkọ', 'nigba', 'fun', 'ninu', 'pada', 'otito', 'iro', 'ohunkohun'
    ];

    const KEYWORDS_ENGLISH = [
        'let', 'if', 'else', 'while', 'for', 'in', 'return', 'true', 'false', 'null'
    ];

    const DOMAINS = [
        'Ogbe', 'Yoruba', 'Irosu', 'Irete', 'Osa', 'Otura', 'Ika', 'Obara', 'Okanran',
        'Ogunda', 'Osa', 'Ika', 'Oturupon', 'Ofun', 'Iwori', 'Odi'
    ];

    function highlightCode() {
        // Find all code blocks that aren't already highlighted
        const blocks = document.querySelectorAll('pre code');

        blocks.forEach(block => {
            if (block.dataset.highlighted) return;

            let html = block.innerHTML;

            // Strings
            html = html.replace(/(".*?")/g, '<span class="string">$1</span>');

            // Comments
            html = html.replace(/(\/\/.*)/g, '<span class="comment">$1</span>');

            // Numbers
            html = html.replace(/\b(\d+)\b/g, '<span class="number">$1</span>');

            // Yoruba Keywords
            KEYWORDS_YORUBA.forEach(kw => {
                const regex = new RegExp(`\\b(${kw})\\b`, 'g');
                html = html.replace(regex, '<span class="keyword yoruba">$1</span>');
            });

            // English Keywords
            KEYWORDS_ENGLISH.forEach(kw => {
                const regex = new RegExp(`\\b(${kw})\\b`, 'g');
                html = html.replace(regex, '<span class="keyword english">$1</span>');
            });

            // Domains (Capitalized)
            DOMAINS.forEach(domain => {
                const regex = new RegExp(`\\b(${domain})\\b`, 'g');
                html = html.replace(regex, '<span class="domain">$1</span>');
            });

            // Functions (word followed by paren)
            html = html.replace(/\b([a-zA-Z_]\w*)\s*\(/g, '<span class="function">$1</span>(');

            block.innerHTML = html;
            block.dataset.highlighted = 'true';
        });
    }

    // Run on load
    document.addEventListener('DOMContentLoaded', highlightCode);

    // Also export for manual calling (e.g. after dynamic content load)
    window.highlightCode = highlightCode;

    // Inject styles if not present
    if (!document.getElementById('syntax-styles')) {
        const style = document.createElement('style');
        style.id = 'syntax-styles';
        style.textContent = `
            .string { color: #a8ff60; }
            .comment { color: #888; font-style: italic; }
            .keyword { color: #ff6b81; font-weight: bold; }
            .number { color: #70a1ff; }
            .domain { color: #ffd700; font-weight: bold; }
            .function { color: #4ade80; }
        `;
        document.head.appendChild(style);
    }
})();
