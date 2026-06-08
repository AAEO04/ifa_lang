/**
 * Client-Side Search for Ifá-Lang Docs
 * Index covers all documented pages.
 */
(function () {
    const searchIndex = [
        // Getting Started
        { title: "Quick Start Guide", href: "getting-started/quickstart.html", content: "installation guide quick start hello world first program" },
        { title: "Installation", href: "getting-started/install.html", content: "install setup requirements dependencies" },
        { title: "Installer Architecture", href: "getting-started/installer.html", content: "installer architecture gui tui" },
        { title: "Hello World", href: "getting-started/hello-world.html", content: "hello world first program basics" },

        // Language
        { title: "Language Syntax", href: "language/syntax.html", content: "syntax grammar keywords operators expressions statements variables types control flow loops functions" },
        { title: "Type System", href: "language/types-crate.html", content: "type system IfaValue variants primitives" },
        { title: "Procedural Macros", href: "language/macros.html", content: "macros Ebo IwaPele Ajose Observable proc-macro" },
        { title: "Language Philosophy", href: "language/philosophy.html", content: "philosophy design 16 odu domains ifa wisdom yoruba" },
        { title: "Iwa Protocol Typing", href: "language/iwa.html", content: "iwa protocol structural typing traits interfaces" },
        { title: "Yanda Memory Model", href: "language/yanda.html", content: "yanda memory ownership deep-copy serialization bincode IfaValueSurrogate" },
        { title: "Ebo Resource Lifecycle", href: "language/ebo.html", content: "ebo resource lifecycle RAII sacrifice cleanup defer scope epoch" },
        { title: "Ewo Taboos Assertions", href: "language/ewo.html", content: "ewo taboo assertion verify runtime constraint invariant" },
        { title: "Ajose Reactivity", href: "language/ajose.html", content: "ajose reactive signal computed effect observer" },
        { title: "Native Array Methods", href: "language/native-methods.html", content: "native map sort filter reduce find every some array methods" },

        // Reference
        { title: "Reference Home", href: "reference/index.html", content: "reference documentation index" },
        { title: "Type System Reference", href: "reference/types.html", content: "types reference IfaValue variants" },
        { title: "Grammar Reference", href: "reference/grammar.html", content: "grammar EBNF syntax rules" },
        { title: "Reserved Words", href: "reference/reserved-words.html", content: "reserved keywords words tokens" },
        { title: "Memory Management", href: "reference/memory.html", content: "memory management garbage collection collector" },
        { title: "Error Reference", href: "reference/errors.html", content: "errors error codes messages" },
        { title: "Comparison with Other Languages", href: "reference/comparison.html", content: "comparison rust python javascript go" },
        { title: "Migrating from Python", href: "reference/migrating-from-python.html", content: "migrate migration python guide" },
        { title: "Migrating from JavaScript", href: "reference/migrating-from-javascript.html", content: "migrate migration javascript js guide" },

        // Domains — 16 canonical
        { title: "Ogbe — System Domain", href: "domains/ogbe.html", content: "ogbe system cli args environment version init" },
        { title: "Oyeku — Exit Domain", href: "domains/oyeku.html", content: "oyeku exit sleep halt termination" },
        { title: "Iwori — Time Domain", href: "domains/iwori.html", content: "iwori time datetime clock timestamp duration" },
        { title: "Odi — File Domain", href: "domains/odi.html", content: "odi file io read write append fs filesystem" },
        { title: "Irosu — Log Domain", href: "domains/irosu.html", content: "irosu log print output console terminal" },
        { title: "Owonrin — Random Domain", href: "domains/owonrin.html", content: "owonrin random uuid shuffle choice" },
        { title: "Obara — Math Domain", href: "domains/obara.html", content: "obara math add multiply pow sqrt" },
        { title: "Okanran — Error Domain", href: "domains/okanran.html", content: "okanran error assert throw panic" },
        { title: "Ogunda — Array Domain", href: "domains/ogunda.html", content: "ogunda array list map filter reduce sort" },
        { title: "Osa — Concurrency Domain", href: "domains/osa.html", content: "osa async await spawn task channel actor" },
        { title: "Ika — String Domain", href: "domains/ika.html", content: "ika string text split join case" },
        { title: "Oturupon — Math II Domain", href: "domains/oturupon.html", content: "oturupon subtract divide modulo negate" },
        { title: "Otura — Network Domain", href: "domains/otura.html", content: "otura network http request fetch download" },
        { title: "Irete — Crypto Domain", href: "domains/irete.html", content: "irete crypto hash sha256 hmac base64" },
        { title: "Ose — UI Domain", href: "domains/ose.html", content: "ose ui tui terminal draw color input" },
        { title: "Ofun — Root Domain", href: "domains/ofun.html", content: "ofun root permissions capabilities reflection debug" },
        { title: "16 Odù Domains Overview", href: "domains/index.html", content: "domains odu overview index all 16" },

        // Infrastructure
        { title: "Infrastructure Overview", href: "infrastructure/index.html", content: "infrastructure overview index" },
        { title: "CPU Infrastructure", href: "infrastructure/infra/cpu.html", content: "cpu parallel rayon par_map par_reduce" },
        { title: "GPU Infrastructure", href: "infrastructure/infra/gpu.html", content: "gpu compute wgpu shader acceleration" },
        { title: "Storage Infrastructure", href: "infrastructure/infra/storage.html", content: "storage persistence kv oduStore set get delete compact" },
        { title: "Kernel/Sys Infrastructure", href: "infrastructure/infra/kernel.html", content: "kernel sys os memory cpu cores uptime" },
        { title: "Shaders Infrastructure", href: "infrastructure/infra/shaders.html", content: "shaders wgsl compute matmul reduce relu softmax" },
        { title: "Infrastructure Modules", href: "infrastructure/infra/index.html", content: "infra modules cpu gpu storage kernel shaders" },
        { title: "Internals", href: "infrastructure/internals/index.html", content: "internals architecture implementation details" },

        // API
        { title: "API Reference", href: "api/api.html", content: "api standard library modules domains all" },
        { title: "API Index", href: "api/index.html", content: "api index reference" },

        // Tutorials
        { title: "Tour: Hello World", href: "tutorials/tour/01-hello.html", content: "tutorial hello world first program basics" },
        { title: "Tour: Variables & Types", href: "tutorials/tour/02-variables.html", content: "tutorial variables types let ayanmo" },
        { title: "Tour: Operators", href: "tutorials/tour/03-operators.html", content: "tutorial operators arithmetic comparison logical" },
        { title: "Tour: Control Flow", href: "tutorials/tour/04-control-flow.html", content: "tutorial control flow if else ti bibi ti nigba while for fun" },
        { title: "Tour: Functions", href: "tutorials/tour/05-functions.html", content: "tutorial functions ese fn parameters return pada" },
        { title: "Tour: Odu Domains", href: "tutorials/tour/06-domains.html", content: "tutorial odu domains 16 api methods" },
        { title: "Tour: Lists & Arrays", href: "tutorials/tour/07-lists.html", content: "tutorial lists arrays ogunda map push pop" },
        { title: "Tour: String Operations", href: "tutorials/tour/08-strings.html", content: "tutorial strings ika split join case" },
        { title: "Tour: File I/O", href: "tutorials/tour/09-files.html", content: "tutorial files odi read write" },
        { title: "Tour: Error Handling", href: "tutorials/tour/10-errors.html", content: "tutorial errors okanran assert panic" },
        { title: "Tour: Parallel Processing", href: "tutorials/tour/11-parallel.html", content: "tutorial parallel cpu par_map par_reduce threads" },
        { title: "Tour: Cryptography", href: "tutorials/tour/12-crypto.html", content: "tutorial crypto irete hash hmac sha256" },
        { title: "Tour: Networking", href: "tutorials/tour/13-network.html", content: "tutorial network otura http request fetch" },
        { title: "Tour: Resource Management", href: "tutorials/tour/14-ebo.html", content: "tutorial ebo resource RAII cleanup" },
        { title: "Tour: Reactivity", href: "tutorials/tour/15-reactivity.html", content: "tutorial reactive ajose signal computed effect" },
        { title: "Tour: Maps & Dictionaries", href: "tutorials/tour/16-maps.html", content: "tutorial maps dictionaries key-value" },
        { title: "Tour: Pattern Matching", href: "tutorials/tour/17-matching.html", content: "tutorial pattern match yan switch" },
        { title: "Tour: Modules & Imports", href: "tutorials/tour/18-modules.html", content: "tutorial modules import iba" },
        { title: "Tour: Safety & Directives", href: "tutorials/tour/19-safety.html", content: "tutorial safety taboo unsafe opon directives" },
        { title: "Tour: Time & Date", href: "tutorials/tour/20-time.html", content: "tutorial time iwori date clock timestamp" },
        { title: "Tour: Randomness", href: "tutorials/tour/21-randomness.html", content: "tutorial random owonrin UUID shuffle" },
        { title: "Tour: Math II (Oturupon)", href: "tutorials/tour/22-oturupon.html", content: "tutorial oturupon subtract divide modulo" },
        { title: "Tour: System Interface", href: "tutorials/tour/23-system.html", content: "tutorial system ogbe oyeku cli env" },
        { title: "Tour: Concurrency & Actors", href: "tutorials/tour/24-concurrency.html", content: "tutorial concurrency osa async actor channel" },
        { title: "Tour: Graphics & UI", href: "tutorials/tour/25-graphics.html", content: "tutorial graphics ose tui terminal ui" },
        { title: "Tour: Security & Capabilities", href: "tutorials/tour/26-security.html", content: "tutorial security ofun capabilities sandbox" },
        { title: "Tour: Building & Packaging", href: "tutorials/tour/27-packaging.html", content: "tutorial build package oja native" },
        { title: "Tour: Static Analysis", href: "tutorials/tour/28-analysis.html", content: "tutorial static analysis babalawo type check lint" },
        { title: "Tour: Developer Tooling", href: "tutorials/tour/29-tooling.html", content: "tutorial tooling lsp formatter debug repl test" },
        { title: "Tour: Embedded & IoT", href: "tutorials/tour/30-embedded.html", content: "tutorial embedded iot esp32 stm32 wasm" },
        { title: "Tour: Memory Management", href: "tutorials/tour/31-memory.html", content: "tutorial memory ebo epochs cycle collector yanda" },
        { title: "Tour: Lambda & Closures", href: "tutorials/tour/32-lambda-move.html", content: "tutorial lambda closure anonymous function move yanda" },
        { title: "Tour: Exceptions", href: "tutorials/tour/33-exceptions.html", content: "tutorial exception try catch finally throw error" },
        { title: "Tour: Defer & Yield", href: "tutorials/tour/34-defer-yield.html", content: "tutorial defer yield cleanup multitasking" },
        { title: "Tour: Advanced Operators", href: "tutorials/tour/35-advanced-operators.html", content: "tutorial power null coalescing set literal interpolated string" },
        { title: "Tour: Type System Deep Dive", href: "tutorials/tour/36-type-system.html", content: "tutorial type system low-level ptr ref effect pure async" },
        { title: "Tour: Alias & Visibility", href: "tutorials/tour/37-alias-visibility.html", content: "tutorial alias assert_type visibility pub private" },
        { title: "Tour: Extended Domains", href: "tutorials/tour/38-extended-domains.html", content: "tutorial extended domains coop ffi opele oracle gpu storage sys" },
        { title: "All Tutorials", href: "tutorials/index.html", content: "tutorials index all guides lessons" },
        { title: "Advanced Tutorials", href: "tutorials/advanced/index.html", content: "advanced tutorials ffi native" },
        { title: "FFI Advanced", href: "tutorials/advanced/ffi.html", content: "ffi native interop c foreign" },
        { title: "Testing Guide", href: "tutorials/testing.html", content: "testing guide unit test" },
        { title: "Debugging Guide", href: "tutorials/debugging.html", content: "debugging guide debug" },
        { title: "Performance Guide", href: "tutorials/performance.html", content: "performance optimization profiling" },
        { title: "Language Tour Index", href: "tutorials/tour/index.html", content: "tour index lessons 1 2 3 4 5 6 7 8" },

        // Development
        { title: "Dev Docs Home", href: "dev/index.html", content: "development docs index" },
        { title: "Crate Map", href: "dev/crate-map.html", content: "crate map architecture crates" },
        { title: "VM Internals", href: "dev/vm.html", content: "vm virtual machine internals interpreter" },
        { title: "Compiler", href: "dev/compiler.html", content: "compiler bytecode translation" },
        { title: "Value System", href: "dev/value-system.html", content: "value system IfaValue NaN-boxing IfaValueSurrogate serialization bincode" },
        { title: "Adding a Domain", href: "dev/adding-a-domain.html", content: "add domain external packages contribute" },
        { title: "Testing Infrastructure", href: "dev/testing.html", content: "testing dev infrastructure" },
        { title: "Memory Model", href: "dev/memory-model.html", content: "memory model IfaGc cycle collector Bacon-Rajan epochs" },
        { title: "Unwired Features", href: "dev/unwired-features.html", content: "unwired features todo unimplemented gaps" },
        { title: "Doc Generation", href: "dev/doc-generation.html", content: "doc generation ifa doc ast parser documentation generator" },

        // Maintainers
        { title: "Maintainer Docs", href: "maintainers/index.html", content: "maintainer documentation index" },
        { title: "Release Process", href: "maintainers/release-process.html", content: "release process version publish" },
        { title: "CI/CD", href: "maintainers/ci-cd.html", content: "ci cd github actions continuous integration" },
        { title: "GC Invariants", href: "maintainers/gc-invariants.html", content: "gc invariants cycle collector safety rules" },

        // Examples
        { title: "Examples Gallery", href: "examples/examples.html", content: "examples code samples gallery" },
        { title: "Playground", href: "examples/playground.html", content: "playground interactive online editor try" },

        // Community
        { title: "Community Hub", href: "community/community.html", content: "community forum chat" },
        { title: "Contributing", href: "community/contributing.html", content: "contributing contribute guide" },
        { title: "Babalawo", href: "community/babalawo.html", content: "babalawo wise elder mentor" },
        { title: "Changelog", href: "community/changelog.html", content: "changelog version history releases" },
        { title: "FAQ", href: "community/faq.html", content: "faq frequently asked questions help" },

        // Tools
        { title: "CLI Tools", href: "tools/cli.html", content: "cli command line ifa run build check test" },
        { title: "IDE Integration", href: "tools/ide-integration.html", content: "ide integration vscode editor" },
        { title: "Sandbox", href: "tools/sandbox.html", content: "sandbox security isolation execution" },

        // Deployment
        { title: "Deployment Guide", href: "deployment/deployment.html", content: "deployment guide" },
        { title: "Embedded Deployment", href: "deployment/embedded.html", content: "embedded esp32 stm32 rp2040 no_std" },
        { title: "Oja Package Manager", href: "deployment/oja.html", content: "oja package manager publish install" },
        { title: "Oja Publishing", href: "deployment/oja-publishing.html", content: "oja publishing package registry" },

        // Other
        { title: "Home", href: "index.html", content: "home documentation index" },
        { title: "404 — Not Found", href: "404.html", content: "404 not found error" }
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
            container.innerHTML = results.slice(0, 20).map(item => `
                <a href="${root}${item.href}" class="search-result-item">
                    <div class="result-title">${item.title}</div>
                </a>
            `).join('');
        }
        container.style.display = 'block';
    }

    const observer = new MutationObserver((mutations) => {
        if (document.getElementById('doc-search')) {
            initSearch();
            observer.disconnect();
        }
    });

    observer.observe(document.body, { childList: true, subtree: true });
})();
