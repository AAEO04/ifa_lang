//! # Odù Wisdom Database
//!
//! Proverbs and meanings for each of the 16 Odù domains.
//! Ported from legacy/src/errors.py ODU_WISDOM dictionary.

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Wisdom entry for a single Odù domain
#[derive(Debug, Clone)]
pub struct OduWisdom {
    pub name: &'static str,
    pub title: &'static str,
    pub meaning: &'static str,
    pub proverbs: &'static [&'static str],
    pub advice: &'static str,
}

/// All 16 Odù domains with their wisdom
pub static ODU_WISDOM: Lazy<HashMap<&'static str, OduWisdom>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    m.insert("OGBE", OduWisdom {
        name: "Ogbè",
        title: "The Light",
        meaning: "Beginnings, initialization, birth",
        proverbs: &[
            "A journey of a thousand miles begins with a single step.",
            "The dawn breaks for those who are prepared.",
            "Light enters where there is an opening.",
        ],
        advice: "Check your initialization. All things must have a proper beginning.",
    });
    
    m.insert("OYEKU", OduWisdom {
        name: "Ọ̀yẹ̀kú",
        title: "The Darkness",
        meaning: "Endings, termination, completion",
        proverbs: &[
            "All rivers flow to the sea.",
            "Even the longest night ends with dawn.",
            "The path that begins must also end.",
        ],
        advice: "Ensure proper termination. Endings must be honored.",
    });
    
    m.insert("IWORI", OduWisdom {
        name: "Ìwòrì",
        title: "The Mirror",
        meaning: "Reflection, iteration, loops",
        proverbs: &[
            "The river does not flow backwards.",
            "What you seek is seeking you.",
            "The mirror shows truth to those who look.",
        ],
        advice: "Check your loop conditions. Cycles must have purpose.",
    });
    
    m.insert("ODI", OduWisdom {
        name: "Òdí",
        title: "The Vessel",
        meaning: "Storage, files, containment",
        proverbs: &[
            "The calabash can only hold what it is given.",
            "An empty vessel makes the most noise.",
            "Guard well what you store.",
        ],
        advice: "Verify your file operations. Vessels must be opened before use and closed after.",
    });
    
    m.insert("IROSU", OduWisdom {
        name: "Ìrosù",
        title: "The Speaker",
        meaning: "Communication, output, expression",
        proverbs: &[
            "Words once spoken cannot be recalled.",
            "The wise speak with purpose.",
            "Let your speech be seasoned with wisdom.",
        ],
        advice: "Check your output format. Communication must be clear.",
    });
    
    m.insert("OWONRIN", OduWisdom {
        name: "Ọ̀wọ́nrín",
        title: "The Chaotic",
        meaning: "Randomness, chance, unpredictability",
        proverbs: &[
            "The wind blows where it wills.",
            "Chaos contains the seed of order.",
            "Expect the unexpected.",
        ],
        advice: "Account for randomness. Chaos must be embraced, not feared.",
    });
    
    m.insert("OBARA", OduWisdom {
        name: "Ọ̀bàrà",
        title: "The King",
        meaning: "Expansion, addition, growth",
        proverbs: &[
            "The tree grows from within.",
            "Small drops fill the ocean.",
            "Growth requires patience and consistency.",
        ],
        advice: "Check your arithmetic. Expansion must respect boundaries.",
    });
    
    m.insert("OKANRAN", OduWisdom {
        name: "Ọ̀kànràn",
        title: "The Troublemaker",
        meaning: "Errors, exceptions, warnings",
        proverbs: &[
            "The squeaking wheel gets the oil.",
            "Problems are opportunities in disguise.",
            "Face your troubles head-on.",
        ],
        advice: "Handle your exceptions. Errors are teachers.",
    });
    
    m.insert("OGUNDA", OduWisdom {
        name: "Ògúndá",
        title: "The Cutter",
        meaning: "Arrays, process control, separation",
        proverbs: &[
            "The machete cuts the path.",
            "Not all that is separated is lost.",
            "To divide is also to organize.",
        ],
        advice: "Check your array bounds. Cutting must be precise.",
    });
    
    m.insert("OSA", OduWisdom {
        name: "Ọ̀sá",
        title: "The Wind",
        meaning: "Control flow, jumps, conditionals",
        proverbs: &[
            "The wind changes direction without announcement.",
            "Flexibility is strength.",
            "Many paths lead to the same destination.",
        ],
        advice: "Verify your conditionals. Flow must have logic.",
    });
    
    m.insert("IKA", OduWisdom {
        name: "Ìká",
        title: "The Constrictor",
        meaning: "Strings, compression, binding",
        proverbs: &[
            "The rope that binds can also free.",
            "Words are the threads that bind meaning.",
            "What is bound together must also be released.",
        ],
        advice: "Check your string operations. Binding must be intentional.",
    });
    
    m.insert("OTURUPON", OduWisdom {
        name: "Òtúúrúpọ̀n",
        title: "The Bearer",
        meaning: "Reduction, subtraction, division",
        proverbs: &[
            "Sharing lightens the load.",
            "Less can be more.",
            "Division creates new wholes.",
        ],
        advice: "Watch for division by zero. Subtraction requires substance.",
    });
    
    m.insert("OTURA", OduWisdom {
        name: "Òtúrá",
        title: "The Messenger",
        meaning: "Network, communication, sending",
        proverbs: &[
            "The messenger is not the message.",
            "Bridges connect distant shores.",
            "News travels faster than the wind.",
        ],
        advice: "Check your network connections. Messages need receivers.",
    });
    
    m.insert("IRETE", OduWisdom {
        name: "Ìrẹtẹ̀",
        title: "The Crusher",
        meaning: "Memory management, garbage collection",
        proverbs: &[
            "Make space for the new by releasing the old.",
            "The granary must be emptied before the harvest.",
            "What is no longer needed becomes burden.",
        ],
        advice: "Free your memory. Release creates space for growth.",
    });
    
    m.insert("OSE", OduWisdom {
        name: "Ọ̀ṣẹ́",
        title: "The Beautifier",
        meaning: "Graphics, display, aesthetics",
        proverbs: &[
            "Beauty speaks without words.",
            "The canvas awaits the artist.",
            "Form follows function.",
        ],
        advice: "Check your display coordinates. Beauty requires precision.",
    });
    
    m.insert("OFUN", OduWisdom {
        name: "Òfún",
        title: "The Creator",
        meaning: "Object creation, inheritance",
        proverbs: &[
            "From nothing, something emerges.",
            "The child inherits from the parent.",
            "Creation is the highest art.",
        ],
        advice: "Verify your object creation. Creation requires intention.",
    });
    
    m
});

/// Error code to Odù domain mapping
pub static ERROR_TO_ODU: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // Initialization errors → Ogbè
    m.insert("UNINITIALIZED", "OGBE");
    m.insert("NULL_REFERENCE", "OGBE");
    m.insert("UNDEFINED_VARIABLE", "OGBE");
    
    // Termination errors → Oyeku
    m.insert("UNCLOSED_RESOURCE", "OYEKU");
    m.insert("ORPHAN_PROCESS", "OYEKU");
    m.insert("INCOMPLETE_SHUTDOWN", "OYEKU");
    
    // Loop errors → Iwori
    m.insert("INFINITE_LOOP", "IWORI");
    m.insert("ITERATOR_EXHAUSTED", "IWORI");
    m.insert("LOOP_INVARIANT_VIOLATED", "IWORI");
    
    // File errors → Odi
    m.insert("FILE_NOT_FOUND", "ODI");
    m.insert("FILE_ALREADY_OPEN", "ODI");
    m.insert("FILE_NOT_CLOSED", "ODI");
    m.insert("PERMISSION_DENIED", "ODI");
    m.insert("PRIVATE_ACCESS", "ODI");
    
    // Output errors → Irosu
    m.insert("FORMAT_ERROR", "IROSU");
    m.insert("OUTPUT_OVERFLOW", "IROSU");
    
    // Random errors → Owonrin
    m.insert("SEED_ERROR", "OWONRIN");
    
    // Math errors → Obara/Oturupon
    m.insert("OVERFLOW", "OBARA");
    m.insert("UNDERFLOW", "OTURUPON");
    m.insert("DIVISION_BY_ZERO", "OTURUPON");
    m.insert("ARITHMETIC_ERROR", "OBARA");
    
    // Exception errors → Okanran
    m.insert("UNHANDLED_EXCEPTION", "OKANRAN");
    m.insert("ASSERTION_FAILED", "OKANRAN");
    m.insert("UNUSED_VARIABLE", "OKANRAN");
    
    // Array errors → Ogunda
    m.insert("INDEX_OUT_OF_BOUNDS", "OGUNDA");
    m.insert("ARRAY_EMPTY", "OGUNDA");
    
    // Control flow errors → Osa
    m.insert("UNREACHABLE_CODE", "OSA");
    m.insert("INVALID_JUMP", "OSA");
    m.insert("MISSING_RETURN", "OSA");
    
    // String errors → Ika
    m.insert("INVALID_ENCODING", "IKA");
    m.insert("STRING_OVERFLOW", "IKA");
    
    // Network errors → Otura
    m.insert("CONNECTION_REFUSED", "OTURA");
    m.insert("TIMEOUT", "OTURA");
    m.insert("NETWORK_UNREACHABLE", "OTURA");
    
    // Memory errors → Irete
    m.insert("MEMORY_LEAK", "IRETE");
    m.insert("DOUBLE_FREE", "IRETE");
    m.insert("OUT_OF_MEMORY", "IRETE");
    
    // Graphics errors → Ose
    m.insert("INVALID_COORDINATES", "OSE");
    m.insert("BUFFER_OVERFLOW", "OSE");
    
    // Object errors → Ofun
    m.insert("TYPE_ERROR", "OFUN");
    m.insert("INHERITANCE_ERROR", "OFUN");
    m.insert("OBJECT_NOT_FOUND", "OFUN");
    
    m
});

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_odu_wisdom_lookup() {
        let ogbe = ODU_WISDOM.get("OGBE").unwrap();
        assert_eq!(ogbe.name, "Ogbè");
        assert_eq!(ogbe.title, "The Light");
        assert!(!ogbe.proverbs.is_empty());
    }
    
    #[test]
    fn test_error_to_odu_mapping() {
        assert_eq!(ERROR_TO_ODU.get("UNDEFINED_VARIABLE"), Some(&"OGBE"));
        assert_eq!(ERROR_TO_ODU.get("DIVISION_BY_ZERO"), Some(&"OTURUPON"));
        assert_eq!(ERROR_TO_ODU.get("FILE_NOT_FOUND"), Some(&"ODI"));
    }
}
