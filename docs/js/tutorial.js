// =============================================================================
// TUTORIAL & LESSONS
// =============================================================================
const lessons = [
    {
        id: 'hello',
        title: 'Hello World',
        icon: '👋',
        desc: 'Print text to the console',
        code: `// Lesson 1: Hello World
// The 'Irosu' domain handles input/output.
Irosu.fo("Hello, World!");
Irosu.fo("Welcome to Ifá-Lang!");
`
    },
    {
        id: 'vars',
        title: 'Variables',
        icon: '📦',
        desc: 'Store data with ayanmọ',
        code: `// Lesson 2: Variables
// Use 'ayanmo' (destiny) to declare variables.

ayanmo name = "Adebayo";
ayanmo age = 25;

Irosu.fo("Name: " + name);
Irosu.fo("Age: " + age);
`
    },
    {
        id: 'math',
        title: 'Math Operations',
        icon: '🧮',
        desc: 'Add, subtract, multiply',
        code: `// Lesson 3: Math (Obara & Oturupon)
// Obara: Addition/Multiplication
// Oturupon: Subtraction/Division

ayanmo a = 10;
ayanmo b = 5;

Irosu.fo(Obara.fikun(a, b));      // 10 + 5
Irosu.fo(Oturupon.din(a, b));     // 10 - 5
Irosu.fo(Obara.isodipupo(a, b));  // 10 * 5
`
    },
    {
        id: 'control',
        title: 'Control Flow',
        icon: '🔀',
        desc: 'If/Else conditions',
        code: `// Lesson 4: Control Flow (Osa)
// 'ti' (if) and 'bibẹkọ' (else)

ayanmo score = 85;

ti score >= 70 {
    Irosu.fo("Distinction! 🌟");
} bibẹkọ {
    Irosu.fo("Try again!");
}
`
    },
    {
        id: 'loops',
        title: 'Loops',
        icon: '🔄',
        desc: 'While loops with nigba',
        code: `// Lesson 5: Loops
// 'nigba' (while/when)

ayanmo count = 5;

nigba count > 0 {
    Irosu.fo("Countdown: " + count);
    count = count - 1;
}

Irosu.fo("Blast off! 🚀");
`
    },
    {
        id: 'functions',
        title: 'Functions',
        icon: '⚙️',
        desc: 'Reusable blocks with ibo',
        code: `// Lesson 6: Functions
// Define functions with 'ibo' (wrapper)

ibo greet(name) {
    pada "Hello, " + name + "!";
}

Irosu.fo(greet("Yemi"));
Irosu.fo(greet("Bolu"));
`
    },
    {
        id: 'arrays',
        title: 'Arrays',
        icon: '📚',
        desc: 'Lists with Ogunda',
        code: `// Lesson 7: Arrays (Ogunda)
// Create and modify lists

ayanmo colors = ["Red", "Green", "Blue"];

Irosu.fo(colors);
Ogunda.so(colors, "Yellow"); // Push
Irosu.fo(colors);
`
    },
    {
        id: 'ose',
        title: 'Ose (Graphics)',
        icon: '🎨',
        desc: 'Draw ASCII art',
        code: `// Lesson 8: Ose Graphics
// Draw shapes and text

Ose.ko(10, 5, "★ Hello ★");
Ose.ila(0, 0, 20, 10, "#");
`
    },
    {
        id: 'opele',
        title: 'Opele Divination',
        icon: '🔮',
        desc: 'Random generation',
        code: `// Lesson 9: Opele
// Cast the oracle

ayanmo odu = Opele.da();
Irosu.fo("Cast result: " + odu);
`
    },
    {
        id: 'completet',
        title: 'Completion',
        icon: '🎓',
        desc: 'You made it!',
        code: `// Congratulations! 🎉
// You've completed the basics of Ifá-Lang.

Irosu.fo("I have learned Ifá-Lang!");
ase;
`
    }
];

let currentLessonIndex = 0;
let completedLessons = JSON.parse(localStorage.getItem('ifa_tutorial_progress') || '[]');

// UI Elements
const tutorialSidebar = document.getElementById('tutorial-sidebar');
const lessonsList = document.getElementById('lessons-list');
const prevBtn = document.getElementById('prev-lesson');
const nextBtn = document.getElementById('next-lesson');
const progressFill = document.getElementById('progress-fill');
const progressText = document.getElementById('progress-text');

// Functions
function initTutorial() {
    renderLessons();
    updateProgress();

    // Check if user has seen tutorial before
    if (!localStorage.getItem('ifa_tutorial_seen')) {
        tutorialSidebar.classList.remove('collapsed');
        localStorage.setItem('ifa_tutorial_seen', 'true');
    } else {
        tutorialSidebar.classList.add('collapsed');
    }
}

function renderLessons() {
    lessonsList.innerHTML = '';
    lessons.forEach((lesson, index) => {
        const isCompleted = completedLessons.includes(lesson.id);
        const isActive = index === currentLessonIndex;

        const item = document.createElement('div');
        item.className = `lesson-item ${isActive ? 'active' : ''} ${isCompleted ? 'completed' : ''}`;
        item.onclick = () => loadLesson(index);
        item.innerHTML = `
            <div class="lesson-icon">${isCompleted ? '✓' : lesson.icon}</div>
            <div class="lesson-details">
                <div class="lesson-title">${lesson.title}</div>
                <div class="lesson-desc">${lesson.desc}</div>
            </div>
        `;
        lessonsList.appendChild(item);
    });
}

function loadLesson(index) {
    currentLessonIndex = index;
    const lesson = lessons[index];
    editor.value = lesson.code;
    updateLineNumbers();

    // Mark as completed
    if (!completedLessons.includes(lesson.id)) {
        completedLessons.push(lesson.id);
        localStorage.setItem('ifa_tutorial_progress', JSON.stringify(completedLessons));
    }

    renderLessons();
    updateProgress();
    updateNavButtons();
}

function updateProgress() {
    const percentage = (completedLessons.length / lessons.length) * 100;
    progressFill.style.width = `${percentage}%`;
    progressText.textContent = `Lesson ${currentLessonIndex + 1} of ${lessons.length}`;
}

function updateNavButtons() {
    prevBtn.disabled = currentLessonIndex === 0;
    nextBtn.disabled = currentLessonIndex === lessons.length - 1;
}

// Event Listeners
document.getElementById('tutorial-toggle').onclick = () => {
    tutorialSidebar.classList.toggle('collapsed');
};

document.getElementById('close-tutorial').onclick = () => {
    tutorialSidebar.classList.add('collapsed');
};

prevBtn.onclick = () => {
    if (currentLessonIndex > 0) loadLesson(currentLessonIndex - 1);
};

nextBtn.onclick = () => {
    if (currentLessonIndex < lessons.length - 1) loadLesson(currentLessonIndex + 1);
};

// Initialize Tutorial
initTutorial();
