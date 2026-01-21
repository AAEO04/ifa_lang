// Enhanced Language Switching for Ifá-Lang Documentation
// Supports individual word switching with aliases

// Word mappings extracted from grammar.pest
const wordMappings = {
    // Keywords
    'import': { yoruba: ['iba', 'ìbà'], english: ['import'] },
    'let': { yoruba: ['ayanmo', 'àyànmọ́'], english: ['let', 'var'] },
    'var': { yoruba: ['ayanmo', 'àyànmọ́'], english: ['let', 'var'] },
    'class': { yoruba: ['odu', 'odù'], english: ['class'] },
    'function': { yoruba: ['ese', 'ẹsẹ'], english: ['fn', 'function', 'def'] },
    'fn': { yoruba: ['ese', 'ẹsẹ'], english: ['fn', 'function', 'def'] },
    'def': { yoruba: ['ese', 'ẹsẹ'], english: ['fn', 'function', 'def'] },
    'if': { yoruba: ['ti', 'bí'], english: ['if'] },
    'else': { yoruba: ['bibẹkọ'], english: ['else'] },
    'while': { yoruba: ['nigba'], english: ['while'] },
    'for': { yoruba: ['fun'], english: ['for'] },
    'in': { yoruba: ['ninu'], english: ['in'] },
    'return': { yoruba: ['pada'], english: ['return'] },
    'end': { yoruba: ['ase', 'àṣẹ'], english: ['end'] },
    'match': { yoruba: ['yàn', 'yán'], english: ['match'] },
    'public': { yoruba: ['gbangba'], english: ['public'] },

    // Memory sizes
    'small': { yoruba: ['kekere', 'kẹ́kẹ́rẹ́'], english: ['small', 'tiny', 'embedded'] },
    'tiny': { yoruba: ['kekere', 'kẹ́kẹ́rẹ́'], english: ['small', 'tiny', 'embedded'] },
    'embedded': { yoruba: ['kekere', 'kẹ́kẹ́rẹ́'], english: ['small', 'tiny', 'embedded'] },
    'medium': { yoruba: ['arinrin', 'àrínrin'], english: ['medium', 'standard', 'default'] },
    'standard': { yoruba: ['arinrin', 'àrínrin'], english: ['medium', 'standard', 'default'] },
    'default': { yoruba: ['arinrin', 'àrínrin'], english: ['medium', 'standard', 'default'] },
    'large': { yoruba: ['nla', 'nlá'], english: ['large', 'big'] },
    'big': { yoruba: ['nla', 'nlá'], english: ['large', 'big'] },
    'unlimited': { yoruba: ['ailopin', 'àìlópin'], english: ['unlimited', 'dynamic', 'max'] },
    'dynamic': { yoruba: ['ailopin', 'àìlópin'], english: ['unlimited', 'dynamic', 'max'] },
    'max': { yoruba: ['ailopin', 'àìlópin'], english: ['unlimited', 'dynamic', 'max'] },

    // Operators
    'and': { yoruba: ['ati'], english: ['and', '&&'] },
    'or': { yoruba: ['tabi'], english: ['or', '||'] },
    'not': { yoruba: ['kii'], english: ['not', '!'] },

    // Special words
    'sacrifice': { yoruba: ['ebo', 'ẹbọ'], english: ['sacrifice'] },
    'taboo': { yoruba: ['èèwọ̀', 'ewọ'], english: ['taboo'] },
    'assert': { yoruba: ['ewo', 'ẹ̀wọ̀'], english: ['assert', 'verify'] },
    'verify': { yoruba: ['ewo', 'ẹ̀wọ̀'], english: ['assert', 'verify'] },
    'true': { yoruba: ['otito'], english: ['true'] },
    'false': { yoruba: ['iro'], english: ['false'] },

    // Console I/O methods
    'fo': { yoruba: ['fo'], english: ['println', 'print'] },
    'println': { yoruba: ['fo'], english: ['println', 'print'] },
    'print': { yoruba: ['fo'], english: ['println', 'print'] },
    'kigbe': { yoruba: ['kigbe'], english: ['shout', 'error'] },
    'shout': { yoruba: ['kigbe'], english: ['shout', 'error'] },
    'error': { yoruba: ['kigbe'], english: ['shout', 'error'] },

    // Crypto methods (Irete)
    'sha256': { yoruba: ['sha256'], english: ['sha256', 'hash'] },
    'hash': { yoruba: ['sha256'], english: ['sha256', 'hash'] },
    'uuid': { yoruba: ['uuid'], english: ['uuid', 'guid'] },
    'guid': { yoruba: ['uuid'], english: ['uuid', 'guid'] },
    'random_bytes': { yoruba: ['random_bytes'], english: ['random_bytes', 'rand_bytes'] },
    'rand_bytes': { yoruba: ['random_bytes'], english: ['random_bytes', 'rand_bytes'] },

    // Network methods (Otura)
    'get': { yoruba: ['get', 'gba'], english: ['get', 'fetch'] },
    'gba': { yoruba: ['get', 'gba'], english: ['get', 'fetch'] },
    'fetch': { yoruba: ['get', 'gba'], english: ['get', 'fetch'] },
    'post': { yoruba: ['post', 'firanṣẹ'], english: ['post', 'send'] },
    'firanṣẹ': { yoruba: ['post', 'firanṣẹ'], english: ['post', 'send'] },
    'send': { yoruba: ['post', 'firanṣẹ'], english: ['post', 'send'] },
    'parse_json': { yoruba: ['parse_json', 'ṣe_json'], english: ['parse_json', 'json_parse'] },
    'ṣe_json': { yoruba: ['parse_json', 'ṣe_json'], english: ['parse_json', 'json_parse'] },
    'json_parse': { yoruba: ['parse_json', 'ṣe_json'], english: ['parse_json', 'json_parse'] },

    // String methods (Ika)
    'len': { yoruba: ['len', 'pẹ̀pẹ̀'], english: ['len', 'length'] },
    'pẹ̀pẹ̀': { yoruba: ['len', 'pẹ̀pẹ̀'], english: ['len', 'length'] },
    'length': { yoruba: ['len', 'pẹ̀pẹ̀'], english: ['len', 'length'] },
    'split': { yoruba: ['split', 'pin'], english: ['split', 'divide'] },
    'pin': { yoruba: ['split', 'pin'], english: ['split', 'divide'] },
    'divide': { yoruba: ['split', 'pin'], english: ['split', 'divide'] },
    'join': { yoruba: ['join', 'dapọ'], english: ['join', 'concat'] },
    'dapọ': { yoruba: ['join', 'dapọ'], english: ['join', 'concat'] },
    'concat': { yoruba: ['join', 'dapọ'], english: ['join', 'concat'] },

    // String case methods (Ika)
    'nla': { yoruba: ['nla'], english: ['uppercase', 'upper'] },
    'kekere': { yoruba: ['kekere'], english: ['lowercase', 'lower'] },

    // Array/List methods (Ogunda)
    'iwon': { yoruba: ['iwon'], english: ['len', 'length'] },
    'fi': { yoruba: ['fi'], english: ['push', 'append'] },
    'mu': { yoruba: ['mu'], english: ['pop'] },
    'pada': { yoruba: ['pada'], english: ['reverse'] },

    // Math methods (Oturupon)
    'sub': { yoruba: ['sub', 'yokuro'], english: ['sub', 'subtract'] },
    'yokuro': { yoruba: ['sub', 'yokuro'], english: ['sub', 'subtract'] },
    'subtract': { yoruba: ['sub', 'yokuro'], english: ['sub', 'subtract'] },
    'div': { yoruba: ['div', 'pin'], english: ['div', 'divide'] },
    'pin': { yoruba: ['div', 'pin'], english: ['div', 'divide'] },
    'divide': { yoruba: ['div', 'pin'], english: ['div', 'divide'] },
    'mod': { yoruba: ['mod', 'iyoku'], english: ['mod', 'remainder'] },
    'iyoku': { yoruba: ['mod', 'iyoku'], english: ['mod', 'remainder'] },
    'remainder': { yoruba: ['mod', 'iyoku'], english: ['mod', 'remainder'] },
    'neg': { yoruba: ['neg', 'odi'], english: ['neg', 'negate'] },
    'odi': { yoruba: ['neg', 'odi'], english: ['neg', 'negate'] },
    'negate': { yoruba: ['neg', 'odi'], english: ['neg', 'negate'] },
    'floor_div': { yoruba: ['floor_div'], english: ['floor_div', 'int_div'] },
    'int_div': { yoruba: ['floor_div'], english: ['floor_div', 'int_div'] },
    'ceil_div': { yoruba: ['ceil_div'], english: ['ceil_div', 'ceil'] },
    'ceil': { yoruba: ['ceil_div'], english: ['ceil_div', 'ceil'] },

    // Random methods (Owonrin)
    'random': { yoruba: ['random'], english: ['random', 'rand'] },
    'rand': { yoruba: ['random'], english: ['random', 'rand'] },
    'shuffle': { yoruba: ['shuffle', 'yipada'], english: ['shuffle', 'mix'] },
    'yipada': { yoruba: ['shuffle', 'yipada'], english: ['shuffle', 'mix'] },
    'mix': { yoruba: ['shuffle', 'yipada'], english: ['shuffle', 'mix'] },
    'pick': { yoruba: ['pick', 'yan'], english: ['pick', 'choose'] },
    'yan': { yoruba: ['pick', 'yan'], english: ['pick', 'choose'] },
    'choose': { yoruba: ['pick', 'yan'], english: ['pick', 'choose'] },

    // Concurrency methods (Osa)
    'sum': { yoruba: ['sum', 'apapọ'], english: ['sum', 'total'] },
    'apapọ': { yoruba: ['sum', 'apapọ'], english: ['sum', 'total'] },
    'total': { yoruba: ['sum', 'apapọ'], english: ['sum', 'total'] },
    'sort': { yoruba: ['sort', 'tọ̀'], english: ['sort', 'order'] },
    'tọ̀': { yoruba: ['sort', 'tọ̀'], english: ['sort', 'order'] },
    'order': { yoruba: ['sort', 'tọ̀'], english: ['sort', 'order'] },
    'min': { yoruba: ['min', 'kere'], english: ['min', 'minimum'] },
    'kere': { yoruba: ['min', 'kere'], english: ['min', 'minimum'] },
    'minimum': { yoruba: ['min', 'kere'], english: ['min', 'minimum'] },
    'max': { yoruba: ['max', 'nla'], english: ['max', 'maximum'] },
    'nla': { yoruba: ['max', 'nla'], english: ['max', 'maximum'] },
    'maximum': { yoruba: ['max', 'nla'], english: ['max', 'maximum'] },
    'threads': { yoruba: ['threads', 'ọ̀rọ̀'], english: ['threads', 'cores'] },
    'ọ̀rọ̀': { yoruba: ['threads', 'ọ̀rọ̀'], english: ['threads', 'cores'] },
    'cores': { yoruba: ['threads', 'ọ̀rọ̀'], english: ['threads', 'cores'] },

    // Terminal methods (Ose)
    'clear': { yoruba: ['clear', 'pa'], english: ['clear', 'cls'] },
    'pa': { yoruba: ['clear', 'pa'], english: ['clear', 'cls'] },
    'cls': { yoruba: ['clear', 'pa'], english: ['clear', 'cls'] },
    'cursor': { yoruba: ['cursor', 'ami'], english: ['cursor', 'move'] },
    'ami': { yoruba: ['cursor', 'ami'], english: ['cursor', 'move'] },
    'move': { yoruba: ['cursor', 'ami'], english: ['cursor', 'move'] },
    'color': { yoruba: ['color', 'awọ'], english: ['color', 'colour'] },
    'awọ': { yoruba: ['color', 'awọ'], english: ['color', 'colour'] },
    'colour': { yoruba: ['color', 'awọ'], english: ['color', 'colour'] },
    'style': { yoruba: ['style', 'ara'], english: ['style', 'format'] },
    'ara': { yoruba: ['style', 'ara'], english: ['style', 'format'] },
    'format': { yoruba: ['style', 'ara'], english: ['style', 'format'] },

    // System methods (Oyeku)
    'sleep': { yoruba: ['sleep', 'sun'], english: ['sleep', 'wait'] },
    'sun': { yoruba: ['sleep', 'sun'], english: ['sleep', 'wait'] },
    'wait': { yoruba: ['sleep', 'sun'], english: ['sleep', 'wait'] },
    'exit': { yoruba: ['exit', 'kuro'], english: ['exit', 'quit'] },
    'kuro': { yoruba: ['exit', 'kuro'], english: ['exit', 'quit'] },
    'quit': { yoruba: ['exit', 'kuro'], english: ['exit', 'quit'] },

    // File I/O methods (Odi)
    'read': { yoruba: ['read', 'kawe'], english: ['read', 'load'] },
    'kawe': { yoruba: ['read', 'kawe'], english: ['read', 'load'] },
    'load': { yoruba: ['read', 'kawe'], english: ['read', 'load'] },
    'write': { yoruba: ['write', 'kọ'], english: ['write', 'save'] },
    'kọ': { yoruba: ['write', 'kọ'], english: ['write', 'save'] },
    'save': { yoruba: ['write', 'kọ'], english: ['write', 'save'] },
    'delete': { yoruba: ['delete', 'pa'], english: ['delete', 'remove'] },
    'remove': { yoruba: ['delete', 'pa'], english: ['delete', 'remove'] },
    'open': { yoruba: ['open', 'si'], english: ['open', 'access'] },
    'si': { yoruba: ['open', 'si'], english: ['open', 'access'] },
    'access': { yoruba: ['open', 'si'], english: ['open', 'access'] },
    'sync': { yoruba: ['sync', 'tọ̀'], english: ['sync', 'flush'] },
    'flush': { yoruba: ['sync', 'tọ̀'], english: ['sync', 'flush'] },

    // Range methods (Iwori)
    'range': { yoruba: ['range', 'ipo'], english: ['range', 'span'] },
    'ipo': { yoruba: ['range', 'ipo'], english: ['range', 'span'] },
    'span': { yoruba: ['range', 'ipo'], english: ['range', 'span'] },

    // Error handling methods (Okanran)
    'try': { yoruba: ['try', 'gbiyanju'], english: ['try', 'attempt'] },
    'gbiyanju': { yoruba: ['try', 'gbiyanju'], english: ['try', 'attempt'] },
    'attempt': { yoruba: ['try', 'gbiyanju'], english: ['try', 'attempt'] },
    'is_error': { yoruba: ['is_error', 'jẹ́aṣiṣe'], english: ['is_error', 'failed'] },
    'jẹ́aṣiṣe': { yoruba: ['is_error', 'jẹ́aṣiṣe'], english: ['is_error', 'failed'] },
    'failed': { yoruba: ['is_error', 'jẹ́aṣiṣe'], english: ['is_error', 'failed'] },
    'message': { yoruba: ['message', 'ifiranṣẹ'], english: ['message', 'msg'] },
    'ifiranṣẹ': { yoruba: ['message', 'ifiranṣẹ'], english: ['message', 'msg'] },
    'msg': { yoruba: ['message', 'ifiranṣẹ'], english: ['message', 'msg'] },

    // Resource management methods (Ebo)
    'new': { yoruba: ['new', 'tuntun'], english: ['new', 'create'] },
    'tuntun': { yoruba: ['new', 'tuntun'], english: ['new', 'create'] },
    'create': { yoruba: ['new', 'tuntun'], english: ['new', 'create'] },
    'dismiss': { yoruba: ['dismiss', 'fiyẹ'], english: ['dismiss', 'cancel'] },
    'fiyẹ': { yoruba: ['dismiss', 'fiyẹ'], english: ['dismiss', 'cancel'] },
    'cancel': { yoruba: ['dismiss', 'fiyẹ'], english: ['dismiss', 'cancel'] },
    'sacrifice': { yoruba: ['sacrifice', 'ẹbọ'], english: ['sacrifice', 'cleanup'] },
    'ẹbọ': { yoruba: ['sacrifice', 'ẹbọ'], english: ['sacrifice', 'cleanup'] },
    'cleanup': { yoruba: ['sacrifice', 'ẹbọ'], english: ['sacrifice', 'cleanup'] },

    // Reactive methods (Signal, Effect, Computed)
    'subscribe': { yoruba: ['subscribe', 'forukọsilẹ'], english: ['subscribe', 'listen'] },
    'forukọsilẹ': { yoruba: ['subscribe', 'forukọsilẹ'], english: ['subscribe', 'listen'] },
    'listen': { yoruba: ['subscribe', 'forukọsilẹ'], english: ['subscribe', 'listen'] },
    'update': { yoruba: ['update', 'ṣe'], english: ['update', 'modify'] },
    'ṣe': { yoruba: ['update', 'ṣe'], english: ['update', 'modify'] },
    'modify': { yoruba: ['update', 'ṣe'], english: ['update', 'modify'] },
    'batch': { yoruba: ['batch', 'akojọ'], english: ['batch', 'group'] },
    'akojọ': { yoruba: ['batch', 'akojọ'], english: ['batch', 'group'] },
    'group': { yoruba: ['batch', 'akojọ'], english: ['batch', 'group'] },

    // Odù domains with English aliases (from lexer.rs)
    // Core 16 Odù
    'irosu': { yoruba: ['Irosu', 'Ìrosù'], english: ['Fmt', 'Log'] },
    'ogbe': { yoruba: ['Ogbe', 'Ogbè'], english: ['Sys', 'Os'] },
    'oyeku': { yoruba: ['Oyeku', 'Ọ̀yẹ̀kú'], english: ['Exit'] },
    'iwori': { yoruba: ['Iwori', 'Ìwòrì'], english: ['Time', 'Datetime'] },
    'odi': { yoruba: ['Odi', 'Òdí'], english: ['Fs', 'Io'] },
    'owonrin': { yoruba: ['Owonrin', 'Ọ̀wọ́nrín'], english: ['Rand'] },
    'obara': { yoruba: ['Obara', 'Ọ̀bàrà'], english: ['Math'] },
    'okanran': { yoruba: ['Okanran', 'Ọ̀kànràn'], english: ['Err', 'Panic'] },
    'ogunda': { yoruba: ['Ogunda', 'Ògúndá'], english: ['Vec', 'List'] },
    'osa': { yoruba: ['Osa', 'Ọ̀sá'], english: ['Async', 'Thread'] },
    'ika': { yoruba: ['Ika', 'Ìká'], english: ['Str', 'String'] },
    'oturupon': { yoruba: ['Oturupon', 'Òtúúrúpọ̀n'], english: ['Div'] },
    'otura': { yoruba: ['Otura', 'Òtúrá'], english: ['Net', 'Http'] },
    'irete': { yoruba: ['Irete', 'Ìrẹtẹ̀'], english: ['Crypto', 'Hash'] },
    'ose': { yoruba: ['Ose', 'Ọ̀ṣẹ́'], english: ['Tui', 'Term'] },
    'ofun': { yoruba: ['Ofun', 'Òfún'], english: ['Perm', 'Auth'] },
    // Pseudo-domains
    'coop': { yoruba: ['Coop', 'Àjọṣe'], english: ['Ffi', 'Bridge'] },
    'opele': { yoruba: ['Opele', 'Ọpẹlẹ'], english: ['Oracle'] },
    // Infrastructure Layer
    'cpu': { yoruba: ['Cpu'], english: ['Parallel'] },
    'gpu': { yoruba: ['Gpu'], english: ['Compute'] },
    'storage': { yoruba: ['Storage'], english: ['Kv', 'Db'] },
    'ohun': { yoruba: ['Ohun'], english: ['Audio', 'Sound'] },
    'fidio': { yoruba: ['Fidio'], english: ['Video', 'Media'] },
    // Application Stacks
    'backend': { yoruba: ['Backend'], english: ['Server'] },
    'frontend': { yoruba: ['Frontend'], english: ['Html', 'Web'] },
    'ml': { yoruba: ['Ml'], english: ['Tensor', 'Ai'] },
    'gamedev': { yoruba: ['GameDev'], english: ['Game', 'Engine'] },
    'iot': { yoruba: ['Iot'], english: ['Gpio', 'Embedded'] },
    // Reverse mappings for English → Yoruba (primary aliases)
    'fmt': { yoruba: ['Irosu'], english: ['Fmt'] },
    'log': { yoruba: ['Irosu'], english: ['Log'] },
    'sys': { yoruba: ['Ogbe'], english: ['Sys'] },
    'os': { yoruba: ['Ogbe'], english: ['Os'] },
    'exit': { yoruba: ['Oyeku'], english: ['Exit'] },
    'time': { yoruba: ['Iwori'], english: ['Time'] },
    'datetime': { yoruba: ['Iwori'], english: ['Datetime'] },
    'fs': { yoruba: ['Odi'], english: ['Fs'] },
    'io': { yoruba: ['Odi'], english: ['Io'] },
    'rand': { yoruba: ['Owonrin'], english: ['Rand'] },
    'math': { yoruba: ['Obara'], english: ['Math'] },
    'err': { yoruba: ['Okanran'], english: ['Err'] },
    'panic': { yoruba: ['Okanran'], english: ['Panic'] },
    'vec': { yoruba: ['Ogunda'], english: ['Vec'] },
    'list': { yoruba: ['Ogunda'], english: ['List'] },
    'async': { yoruba: ['Osa'], english: ['Async'] },
    'thread': { yoruba: ['Osa'], english: ['Thread'] },
    'str': { yoruba: ['Ika'], english: ['Str'] },
    'string': { yoruba: ['Ika'], english: ['String'] },
    'div': { yoruba: ['Oturupon'], english: ['Div'] },
    'net': { yoruba: ['Otura'], english: ['Net'] },
    'http': { yoruba: ['Otura'], english: ['Http'] },
    'crypto': { yoruba: ['Irete'], english: ['Crypto'] },
    'hash': { yoruba: ['Irete'], english: ['Hash'] },
    'tui': { yoruba: ['Ose'], english: ['Tui'] },
    'term': { yoruba: ['Ose'], english: ['Term'] },
    'perm': { yoruba: ['Ofun'], english: ['Perm'] },
    'auth': { yoruba: ['Ofun'], english: ['Auth'] },
    'ffi': { yoruba: ['Coop'], english: ['Ffi'] },
    'bridge': { yoruba: ['Coop'], english: ['Bridge'] },
    'parallel': { yoruba: ['Cpu'], english: ['Parallel'] },
    'compute': { yoruba: ['Gpu'], english: ['Compute'] },
    'kv': { yoruba: ['Storage'], english: ['Kv'] },
    'db': { yoruba: ['Storage'], english: ['Db'] },
    'audio': { yoruba: ['Ohun'], english: ['Audio'] },
    'sound': { yoruba: ['Ohun'], english: ['Sound'] },
    'video': { yoruba: ['Fidio'], english: ['Video'] },
    'media': { yoruba: ['Fidio'], english: ['Media'] },
    'server': { yoruba: ['Backend'], english: ['Server'] },
    'html': { yoruba: ['Frontend'], english: ['Html'] },
    'web': { yoruba: ['Frontend'], english: ['Web'] },
    'tensor': { yoruba: ['Ml'], english: ['Tensor'] },
    'ai': { yoruba: ['Ml'], english: ['Ai'] },
    'game': { yoruba: ['GameDev'], english: ['Game'] },
    'engine': { yoruba: ['GameDev'], english: ['Engine'] },
    'gpio': { yoruba: ['Iot'], english: ['Gpio'] },
    'embedded': { yoruba: ['Iot'], english: ['Embedded'] }
};

// Create reverse lookup for efficient switching
const reverseLookup = {};
Object.keys(wordMappings).forEach(key => {
    const mapping = wordMappings[key];
    [...mapping.yoruba, ...mapping.english].forEach(word => {
        reverseLookup[word.toLowerCase()] = {
            canonical: key,
            yoruba: mapping.yoruba,
            english: mapping.english
        };
    });
});

// Current language state
let currentLanguage = 'yoruba';

// Enhanced language switching function
function switchLanguage(targetLang = null) {
    currentLanguage = targetLang || (currentLanguage === 'yoruba' ? 'english' : 'yoruba');

    document.querySelectorAll('.switchable-word').forEach(element => {
        const word = element.textContent.toLowerCase();
        const mapping = reverseLookup[word];

        if (mapping) {
            const alternatives = currentLanguage === 'yoruba' ? mapping.yoruba : mapping.english;
            element.textContent = alternatives[0]; // Use first alternative
            element.setAttribute('data-lang', currentLanguage);
        }
    });

    // Update toggle buttons
    document.querySelectorAll('.toggle-yoruba, .toggle-english').forEach(btn => {
        btn.classList.toggle('active',
            (btn.classList.contains('toggle-yoruba') && currentLanguage === 'yoruba') ||
            (btn.classList.contains('toggle-english') && currentLanguage === 'english')
        );
    });

    return currentLanguage;
}

// Individual word switching on click
function switchWord(element) {
    const word = element.textContent.toLowerCase();
    const mapping = reverseLookup[word];

    if (mapping) {
        const currentLang = element.getAttribute('data-lang') || 'yoruba';
        const alternatives = currentLang === 'yoruba' ? mapping.english : mapping.yoruba;
        const newWord = alternatives[0];
        const newLang = currentLang === 'yoruba' ? 'english' : 'yoruba';

        element.textContent = newWord;
        element.setAttribute('data-lang', newLang);

        // Add animation effect
        element.style.transition = 'all 0.3s ease';
        element.style.transform = 'scale(1.1)';
        setTimeout(() => {
            element.style.transform = 'scale(1)';
        }, 150);
    }
}

// Initialize switchable words in code blocks
function initializeSwitchableWords() {
    document.querySelectorAll('pre code').forEach(codeBlock => {
        // Skip if already processed or if it's part of the old dual-block system
        if (codeBlock.querySelector('.switchable-word') ||
            codeBlock.closest('.code-yoruba') ||
            codeBlock.closest('.code-english')) {
            return;
        }

        const html = codeBlock.innerHTML;
        const words = html.split(/(\s+|[{}();.,\[\]"])/);

        const processedWords = words.map(word => {
            const trimmed = word.trim();
            const mapping = reverseLookup[trimmed.toLowerCase()];

            if (mapping && trimmed && !/^\s+$/.test(trimmed)) {
                return `<span class="switchable-word" data-lang="${currentLanguage}" onclick="switchWord(this)">${trimmed}</span>`;
            }
            return word;
        });

        codeBlock.innerHTML = processedWords.join('');
    });
}

// Enhanced initialization that also adds toggle buttons to code examples without them
function enhanceAllCodeExamples() {
    // First, add toggle buttons to code examples that don't have them
    document.querySelectorAll('.code-example, .example').forEach(container => {
        const header = container.querySelector('.example-header, .code-header');
        if (header && !header.querySelector('.lang-toggle')) {
            const toggleDiv = document.createElement('div');
            toggleDiv.className = 'lang-toggle';
            toggleDiv.innerHTML = `
                <button class="toggle-yoruba ${currentLanguage === 'yoruba' ? 'active' : ''}" onclick="setLang('yoruba')">Yoruba</button>
                <button class="toggle-english ${currentLanguage === 'english' ? 'active' : ''}" onclick="setLang('english')">English</button>
            `;
            header.appendChild(toggleDiv);
        }
    });

    // Then initialize switchable words
    initializeSwitchableWords();
}

// Enhanced setLang function to maintain compatibility with both systems
function setLang(lang) {
    currentLanguage = lang;

    // Handle new individual word switching system
    document.querySelectorAll('.switchable-word').forEach(element => {
        const word = element.textContent.toLowerCase();
        const mapping = reverseLookup[word];

        if (mapping) {
            const alternatives = currentLanguage === 'yoruba' ? mapping.yoruba : mapping.english;
            element.textContent = alternatives[0];
            element.setAttribute('data-lang', currentLanguage);
        }
    });

    // Handle old dual-code block system
    document.querySelectorAll('.code-yoruba').forEach(el => {
        el.style.display = lang === 'yoruba' ? 'block' : 'none';
    });
    document.querySelectorAll('.code-english').forEach(el => {
        el.style.display = lang === 'english' ? 'block' : 'none';
    });

    // Update toggle buttons
    document.querySelectorAll('.toggle-yoruba, .toggle-english').forEach(btn => {
        btn.classList.toggle('active',
            (btn.classList.contains('toggle-yoruba') && currentLanguage === 'yoruba') ||
            (btn.classList.contains('toggle-english') && currentLanguage === 'english')
        );
    });
}

// Auto-initialize when DOM is ready
document.addEventListener('DOMContentLoaded', function () {
    enhanceAllCodeExamples();
});

// Export for use in other files
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { switchLanguage, switchWord, setLang, wordMappings };
}
