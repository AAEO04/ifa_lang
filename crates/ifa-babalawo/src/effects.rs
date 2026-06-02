use ifa_types::ast::Effect;
use ifa_types::IfaError;

#[derive(Debug)]
pub struct EffectChecker {
    pub current_effects: Vec<Effect>,
    pub errors: Vec<IfaError>,
    pub in_function: bool,
}

impl EffectChecker {
    pub fn new() -> Self {
        Self {
            current_effects: Vec::new(),
            errors: Vec::new(),
            in_function: false,
        }
    }

    pub fn enter_function(&mut self, effects: Vec<Effect>) {
        self.current_effects = effects;
        self.in_function = true;
    }

    pub fn leave_function(&mut self) {
        self.current_effects.clear();
        self.in_function = false;
    }

    pub fn check_call(&mut self, callee_effects: &[Effect], file: &str, line: usize, column: usize) {
        if !self.in_function {
            return;
        }

        // Enforce that a Pure function cannot call functions with effects
        if self.current_effects.contains(&Effect::Pure) {
            for effect in callee_effects {
                if effect != &Effect::Pure {
                    self.errors.push(IfaError::Custom(format!(
                        "Pure function cannot call function with effect {:?} at {}:{}:{}",
                        effect, file, line, column
                    )));
                }
            }
        }

        // More general effect containment rules can go here
        for effect in callee_effects {
            if effect != &Effect::Pure && !self.current_effects.contains(effect) && !self.current_effects.contains(&Effect::Impure) {
                // Warning or error depending on strictness
                // Let's record an error for unhandled effect
                self.errors.push(IfaError::Custom(format!(
                    "Function is missing effect declaration {:?} for call at {}:{}:{}",
                    effect, file, line, column
                )));
            }
        }
    }
}

pub fn domain_effects(domain: ifa_types::domain::OduDomain) -> Vec<Effect> {
    match domain {
        ifa_types::domain::OduDomain::Osa => vec![Effect::Async],
        ifa_types::domain::OduDomain::Otura => vec![Effect::Network],
        ifa_types::domain::OduDomain::Odi | ifa_types::domain::OduDomain::Storage => vec![Effect::FileIO],
        ifa_types::domain::OduDomain::Ofun | ifa_types::domain::OduDomain::Sys | ifa_types::domain::OduDomain::Coop => vec![Effect::Impure],
        _ => if domain.has_side_effects() { vec![Effect::Impure] } else { vec![Effect::Pure] },
    }
}
