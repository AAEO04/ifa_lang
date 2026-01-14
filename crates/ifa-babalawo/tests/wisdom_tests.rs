//! Comprehensive tests for ifa-babalawo wisdom system

use ifa_babalawo::*;

mod wisdom_engine_tests {
    use super::*;

    #[test]
    fn test_wisdom_creation() {
        let wisdom = Wisdom::new();
        
        // New wisdom should be empty
        assert_eq!(wisdom.get_proverb_count(), 0);
        assert_eq!(wisdom.get_teaching_count(), 0);
        assert!(wisdom.is_empty());
    }

    #[test]
    fn test_add_proverb() {
        let mut wisdom = Wisdom::new();
        
        let proverb = Proverb {
            id: "test_001".to_string(),
            text: "A kì í fi ẹyìn àgbà r'ẹsìn".to_string(),
            translation: "We don't use the back of an elder as a drum".to_string(),
            meaning: "Respect your elders".to_string(),
            odu: Odu::Ogbe,
            context: ProverbContext::Social,
        };
        
        wisdom.add_proverb(proverb.clone());
        
        assert_eq!(wisdom.get_proverb_count(), 1);
        assert!(!wisdom.is_empty());
        
        let retrieved = wisdom.get_proverb("test_001").unwrap();
        assert_eq!(retrieved.text, proverb.text);
        assert_eq!(retrieved.translation, proverb.translation);
    }

    #[test]
    fn test_add_teaching() {
        let mut wisdom = Wisdom::new();
        
        let teaching = Teaching {
            id: "teach_001".to_string(),
            title: "The Importance of Patience".to_string(),
            content: "Patience is the key to wisdom in Ifá".to_string(),
            odu: Odu::Iwori,
            principles: vec![
                "Wait for the right time".to_string(),
                "Don't rush decisions".to_string(),
            ],
            examples: vec![
                "The farmer waits for the right season to plant".to_string(),
            ],
        };
        
        wisdom.add_teaching(teaching.clone());
        
        assert_eq!(wisdom.get_teaching_count(), 1);
        
        let retrieved = wisdom.get_teaching("teach_001").unwrap();
        assert_eq!(retrieved.title, teaching.title);
        assert_eq!(retrieved.odu, teaching.odu);
    }

    #[test]
    fn test_search_proverbs() {
        let mut wisdom = Wisdom::new();
        
        // Add test proverbs
        wisdom.add_proverb(Proverb {
            id: "p1".to_string(),
            text: "Proverb about patience".to_string(),
            translation: "Translation about patience".to_string(),
            meaning: "Be patient".to_string(),
            odu: Odu::Ogbe,
            context: ProverbContext::Social,
        });
        
        wisdom.add_proverb(Proverb {
            id: "p2".to_string(),
            text: "Proverb about wisdom".to_string(),
            translation: "Translation about wisdom".to_string(),
            meaning: "Seek wisdom".to_string(),
            odu: Odu::Oyeku,
            context: ProverbContext::Spiritual,
        });
        
        // Search by text
        let patience_results = wisdom.search_proverbs("patience");
        assert_eq!(patience_results.len(), 1);
        assert_eq!(patience_results[0].id, "p1");
        
        // Search by meaning
        let wisdom_results = wisdom.search_by_meaning("wisdom");
        assert_eq!(wisdom_results.len(), 1);
        assert_eq!(wisdom_results[0].id, "p2");
        
        // Search by Odu
        let ogbe_results = wisdom.search_by_odu(Odu::Ogbe);
        assert_eq!(ogbe_results.len(), 1);
        assert_eq!(ogbe_results[0].id, "p1");
    }

    #[test]
    fn test_wisdom_filtering() {
        let mut wisdom = Wisdom::new();
        
        // Add proverbs with different contexts
        wisdom.add_proverb(Proverb {
            id: "social".to_string(),
            text: "Social proverb".to_string(),
            translation: "Social translation".to_string(),
            meaning: "Social meaning".to_string(),
            odu: Odu::Ogbe,
            context: ProverbContext::Social,
        });
        
        wisdom.add_proverb(Proverb {
            id: "spiritual".to_string(),
            text: "Spiritual proverb".to_string(),
            translation: "Spiritual translation".to_string(),
            meaning: "Spiritual meaning".to_string(),
            odu: Odu::Oyeku,
            context: ProverbContext::Spiritual,
        });
        
        // Filter by context
        let social_proverbs = wisdom.filter_by_context(ProverbContext::Social);
        assert_eq!(social_proverbs.len(), 1);
        assert_eq!(social_proverbs[0].id, "social");
        
        let spiritual_proverbs = wisdom.filter_by_context(ProverbContext::Spiritual);
        assert_eq!(spiritual_proverbs.len(), 1);
        assert_eq!(spiritual_proverbs[0].id, "spiritual");
    }
}

mod odu_interpretation_tests {
    use super::*;

    #[test]
    fn test_odu_creation() {
        let odu = OduInterpretation::new(Odu::Ogbe);
        
        assert_eq!(odu.odu(), Odu::Ogbe);
        assert!(odu.verses().is_empty());
        assert!(odu.interpretations().is_empty());
    }

    #[test]
    fn test_add_verse() {
        let mut odu = OduInterpretation::new(Odu::Ogbe);
        
        let verse = Verse {
            number: 1,
            text: "Ogbe di kógun, kógun ò rà".to_string(),
            translation: "Ogbe says go to war, war is not profitable".to_string(),
            explanation: "Sometimes going to war brings no benefit".to_string(),
        };
        
        odu.add_verse(verse.clone());
        
        assert_eq!(odu.verses().len(), 1);
        let retrieved = odu.get_verse(1).unwrap();
        assert_eq!(retrieved.text, verse.text);
    }

    #[test]
    fn test_add_interpretation() {
        let mut odu = OduInterpretation::new(Odu::Ogbe);
        
        let interpretation = Interpretation {
            context: InterpretationContext::Personal,
            situation: "Facing a difficult decision".to_string(),
            guidance: "Consult with elders before acting".to_string(),
            warning: "Acting hastily may lead to regret".to_string(),
            outcome: "Wisdom through patience".to_string(),
        };
        
        odu.add_interpretation(interpretation.clone());
        
        assert_eq!(odu.interpretations().len(), 1);
        let retrieved = odu.get_interpretation(InterpretationContext::Personal).unwrap();
        assert_eq!(retrieved.situation, interpretation.situation);
    }

    #[test]
    fn test_odu_relationships() {
        let mut ogbe = OduInterpretation::new(Odu::Ogbe);
        let mut oyeku = OduInterpretation::new(Odu::Oyeku);
        
        // Establish relationship
        ogbe.add_companion(Odu::Oyeku);
        oyeku.add_companion(Odu::Ogbe);
        
        assert!(ogbe.is_companion_with(Odu::Oyeku));
        assert!(oyeku.is_companion_with(Odu::Ogbe));
        
        // Test opposite relationship
        assert!(!ogbe.is_opposite_of(Odu::Oyeku));
        
        // Add opposite
        ogbe.add_opposite(Odu::Oyeku);
        assert!(ogbe.is_opposite_of(Odu::Oyeku));
    }
}

mod consultation_tests {
    use super::*;

    #[test]
    fn test_consultation_creation() {
        let consultation = Consultation::new();
        
        assert!(consultation.questions().is_empty());
        assert!(consultation.answers().is_empty());
        assert!(consultation.timestamp() <= std::time::SystemTime::now());
    }

    #[test]
    fn test_add_question() {
        let mut consultation = Consultation::new();
        
        let question = Question {
            id: "q1".to_string(),
            text: "Should I take this job offer?".to_string(),
            category: QuestionCategory::Career,
            urgency: Urgency::Normal,
            context: "I have two job offers".to_string(),
        };
        
        consultation.add_question(question.clone());
        
        assert_eq!(consultation.questions().len(), 1);
        let retrieved = consultation.get_question("q1").unwrap();
        assert_eq!(retrieved.text, question.text);
    }

    #[test]
    fn test_provide_answer() {
        let mut consultation = Consultation::new();
        
        let question = Question {
            id: "q1".to_string(),
            text: "Should I take this job offer?".to_string(),
            category: QuestionCategory::Career,
            urgency: Urgency::Normal,
            context: "I have two job offers".to_string(),
        };
        
        consultation.add_question(question);
        
        let answer = Answer {
            question_id: "q1".to_string(),
            odu: Odu::Ogbe,
            verses: vec!["Verse about decision making".to_string()],
            interpretation: "Consider both options carefully".to_string(),
            advice: "Seek counsel from trusted advisors".to_string(),
            confidence: 0.85,
        };
        
        consultation.provide_answer(answer.clone());
        
        assert_eq!(consultation.answers().len(), 1);
        let retrieved = consultation.get_answer("q1").unwrap();
        assert_eq!(retrieved.odu, Odu::Ogbe);
        assert_eq!(retrieved.confidence, 0.85);
    }

    #[test]
    fn test_consultation_completeness() {
        let mut consultation = Consultation::new();
        
        let question = Question {
            id: "q1".to_string(),
            text: "Test question".to_string(),
            category: QuestionCategory::General,
            urgency: Urgency::Normal,
            context: "Test context".to_string(),
        };
        
        consultation.add_question(question);
        
        // Should not be complete without answer
        assert!(!consultation.is_complete());
        
        let answer = Answer {
            question_id: "q1".to_string(),
            odu: Odu::Ogbe,
            verses: vec!["Test verse".to_string()],
            interpretation: "Test interpretation".to_string(),
            advice: "Test advice".to_string(),
            confidence: 0.9,
        };
        
        consultation.provide_answer(answer);
        
        // Should be complete with answer
        assert!(consultation.is_complete());
    }
}

mod divination_tests {
    use super::*;

    #[test]
    fn test_divination_process() {
        let diviner = Diviner::new();
        
        // Mock divination
        let result = diviner.cast();
        
        assert!(result.primary_odu() != Odu::Unknown);
        assert!(result.secondary_odu() != Odu::Unknown);
        assert!(result.confidence() > 0.0);
        assert!(result.confidence() <= 1.0);
    }

    #[test]
    fn test_divination_interpretation() {
        let diviner = Diviner::new();
        
        let result = DivinationResult {
            primary_odu: Odu::Ogbe,
            secondary_odu: Odu::Oyeku,
            confidence: 0.9,
            timestamp: std::time::SystemTime::now(),
            verses: vec!["Test verse".to_string()],
            interpretation: "Test interpretation".to_string(),
        };
        
        let interpretation = diviner.interpret(&result);
        
        assert!(!interpretation.is_empty());
        assert!(interpretation.contains("Ogbe") || interpretation.contains("Oyeku"));
    }

    #[test]
    fn test_divination_history() {
        let mut diviner = Diviner::new();
        
        // Perform multiple divinations
        let result1 = diviner.cast();
        let result2 = diviner.cast();
        let result3 = diviner.cast();
        
        let history = diviner.get_history();
        assert_eq!(history.len(), 3);
        
        // Check order (most recent first)
        assert_eq!(history[0].timestamp(), result3.timestamp());
        assert_eq!(history[1].timestamp(), result2.timestamp());
        assert_eq!(history[2].timestamp(), result1.timestamp());
    }

    #[test]
    fn test_divination_patterns() {
        let mut diviner = Diviner::new();
        
        // Cast many times to test patterns
        let mut odu_counts = std::collections::HashMap::new();
        
        for _ in 0..100 {
            let result = diviner.cast();
            *odu_counts.entry(result.primary_odu()).or_insert(0) += 1;
        }
        
        // Should have variety of Odu
        assert!(odu_counts.len() > 10);
        
        // Each Odu should appear reasonable number of times
        for count in odu_counts.values() {
            assert!(*count > 0);
            assert!(*count < 50); // No single Odu should dominate
        }
    }
}

mod ritual_tests {
    use super::*;

    #[test]
    fn test_ritual_creation() {
        let ritual = Ritual::new(
            "Morning Prayer".to_string(),
            RitualType::Prayer,
            Odu::Ogbe,
        );
        
        assert_eq!(ritual.name(), "Morning Prayer");
        assert_eq!(ritual.ritual_type(), RitualType::Prayer);
        assert_eq!(ritual.primary_odu(), Odu::Ogbe);
        assert!(ritual.steps().is_empty());
    }

    #[test]
    fn test_add_ritual_step() {
        let mut ritual = Ritual::new(
            "Test Ritual".to_string(),
            RitualType::Offering,
            Odu::Ogbe,
        );
        
        let step = RitualStep {
            order: 1,
            description: "Light white candle".to_string(),
            action: "Light a white candle and place on altar".to_string(),
            materials: vec!["White candle".to_string(), "Matches".to_string()],
            timing: StepTiming::Beginning,
            duration: Some(std::time::Duration::from_secs(60)),
        };
        
        ritual.add_step(step.clone());
        
        assert_eq!(ritual.steps().len(), 1);
        let retrieved = ritual.get_step(1).unwrap();
        assert_eq!(retrieved.description, step.description);
    }

    #[test]
    fn test_ritual_validation() {
        let mut ritual = Ritual::new(
            "Test Ritual".to_string(),
            RitualType::Offering,
            Odu::Ogbe,
        );
        
        // Empty ritual should be invalid
        assert!(!ritual.is_valid());
        
        // Add required steps
        ritual.add_step(RitualStep {
            order: 1,
            description: "Preparation".to_string(),
            action: "Prepare the space".to_string(),
            materials: vec!["Clean cloth".to_string()],
            timing: StepTiming::Beginning,
            duration: None,
        });
        
        ritual.add_step(RitualStep {
            order: 2,
            description: "Main action".to_string(),
            action: "Perform the main ritual".to_string(),
            materials: vec!["Offering items".to_string()],
            timing: StepTiming::Middle,
            duration: Some(std::time::Duration::from_secs(300)),
        });
        
        ritual.add_step(RitualStep {
            order: 3,
            description: "Closing".to_string(),
            action: "Close the ritual".to_string(),
            materials: vec![],
            timing: StepTiming::End,
            duration: None,
        });
        
        // Should be valid with proper structure
        assert!(ritual.is_valid());
    }

    #[test]
    fn test_ritual_execution() {
        let mut ritual = Ritual::new(
            "Simple Prayer".to_string(),
            RitualType::Prayer,
            Odu::Ogbe,
        );
        
        ritual.add_step(RitualStep {
            order: 1,
            description: "Begin prayer".to_string(),
            action: "Say opening prayer".to_string(),
            materials: vec![],
            timing: StepTiming::Beginning,
            duration: None,
        });
        
        let executor = RitualExecutor::new();
        let result = executor.execute(&ritual);
        
        assert!(result.is_success());
        assert!(result.execution_time() > std::time::Duration::from_secs(0));
    }
}

mod taboo_tests {
    use super::*;

    #[test]
    fn test_taboo_creation() {
        let taboo = Taboo::new(
            "No whistling at night".to_string(),
            TabooType::Behavioral,
            Odu::Oyeku,
        );
        
        assert_eq!(taboo.description(), "No whistling at night");
        assert_eq!(taboo.taboo_type(), TabooType::Behavioral);
        assert_eq!(taboo.associated_odu(), Odu::Oyeku);
    }

    #[test]
    fn test_taboo_consequences() {
        let mut taboo = Taboo::new(
            "Eating forbidden food".to_string(),
            TabooType::Dietary,
            Odu::Iwori,
        );
        
        taboo.add_consequence(TabooConsequence {
            severity: ConsequenceSeverity::Minor,
            description: "Stomach upset".to_string(),
            remedy: "Herbal medicine".to_string(),
            duration: Some(std::time::Duration::from_secs(86400)), // 1 day
        });
        
        let consequences = taboo.consequences();
        assert_eq!(consequences.len(), 1);
        assert_eq!(consequences[0].severity, ConsequenceSeverity::Minor);
    }

    #[test]
    fn test_taboo_exceptions() {
        let mut taboo = Taboo::new(
            "No speaking during ceremony".to_string(),
            TabooType::Ceremonial,
            Odu::Ogunda,
        );
        
        taboo.add_exception("Priest may speak".to_string());
        taboo.add_exception("Emergency situations".to_string());
        
        let exceptions = taboo.exceptions();
        assert_eq!(exceptions.len(), 2);
        assert!(exceptions.contains(&"Priest may speak".to_string()));
    }

    #[test]
    fn test_taboo_checking() {
        let taboo_system = TabooSystem::new();
        
        // Add some taboos
        taboo_system.add_taboo(Taboo::new(
            "No work on sacred day".to_string(),
            TabooType::Temporal,
            Odu::Osa,
        ));
        
        taboo_system.add_taboo(Taboo::new(
            "No eating pork".to_string(),
            TabooType::Dietary,
            Odu::Otura,
        ));
        
        // Check violations
        assert!(taboo_system.is_violation("work", Some(Odu::Osa)));
        assert!(taboo_system.is_violation("eat pork", Some(Odu::Otura)));
        assert!(!taboo_system.is_violation("pray", Some(Odu::Ogbe)));
    }
}
