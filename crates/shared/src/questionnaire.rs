//! Core psychometric data structures, TGA scoring engine, and administration protocols
//! for Revisited IPIP-NEO.
//!
//! # Tridimensional / Three-Tier Psychometric Model (TGA)
//! - **Hierarchy:** 221 optimized items map simultaneously onto 28 primary [`Facet`]s,
//!   6 higher-order [`Trait`]s, and 3 broad [`MetaTrait`] dimensions via signed item weights.
//! - **Normalized Scores:** Each construct accumulator maps weighted Likert responses `[-1.0, 1.0]`
//!   onto `normalized_score = sum(score_i * w_i) / sum(|w_i|)`.
//! - **Precision (SE):** Scale-level standard error on `[-1.0, 1.0]` is estimated as
//!   `SE = (sqrt(sum(w_i^2)) / sum(|w_i|)) * 0.5`.
//! - **Population Norming:** [`NormSet`] converts normalized scores to z-scores and Gaussian
//!   percentile estimates; the default [`NormSet::theoretical_tga`] uses a synthetic
//!   zero-centered prior pending empirical reference samples.
//! - **Administration Modes:** [`AdministrationMode::Full221`] administers all 221 items, while
//!   [`AdministrationMode::Rapid112`] selects a facet-balanced 112-item short protocol without
//!   discarding previously entered answers.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt;

/// Embed source CSV datasets at compile time.
pub const DISTILLED_KEY_CSV: &str = include_str!("../../../Distilled Key.csv");
pub const OPTIMIZED_KEYS_CSV: &str = include_str!("../../../Optimized_Keys.csv");

/// Continuous response options mapped into range [-1.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(i8)]
pub enum Response {
    StronglyDisagree = -2,
    Disagree = -1,
    Neutral = 0,
    Agree = 1,
    StronglyAgree = 2,
}

impl Response {
    #[inline]
    pub const fn to_score(self) -> f32 {
        match self {
            Self::StronglyDisagree => -1.0,
            Self::Disagree => -0.5,
            Self::Neutral => 0.0,
            Self::Agree => 0.5,
            Self::StronglyAgree => 1.0,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::StronglyDisagree => "Strongly Disagree",
            Self::Disagree => "Disagree",
            Self::Neutral => "Neutral",
            Self::Agree => "Agree",
            Self::StronglyAgree => "Strongly Agree",
        }
    }

    pub const fn shortcut(self) -> &'static str {
        match self {
            Self::StronglyDisagree => "1",
            Self::Disagree => "2",
            Self::Neutral => "3",
            Self::Agree => "4",
            Self::StronglyAgree => "5",
        }
    }
}

pub trait Aspect: Copy + Eq + std::hash::Hash + fmt::Debug + 'static {
    fn display_name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Facet {
    Anxiety,
    Gregariousness,
    Trust,
    SelfEfficacy,
    Anger,
    Fairness,
    Orderliness,
    Dominance,
    Emotionality,
    Adventurousness,
    Determination,
    ExcitementSeeking,
    Intellect,
    AttentionSeeking,
    Cheerfulness,
    Liberalism,
    ArtisticInterests,
    Empathy,
    WorkEthic,
    Cautiousness,
    Manipulativeness,
    Humility,
    Introspection,
    Honesty,
    Immoderation,
    SelfDiscipline,
    Recklessness,
    Calmness,
}

impl Aspect for Facet {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Anxiety => "Anxiety",
            Self::Gregariousness => "Gregariousness",
            Self::Trust => "Trust",
            Self::SelfEfficacy => "Self-Efficacy",
            Self::Anger => "Anger",
            Self::Fairness => "Fairness",
            Self::Orderliness => "Orderliness",
            Self::Dominance => "Dominance",
            Self::Emotionality => "Emotionality",
            Self::Adventurousness => "Adventurousness",
            Self::Determination => "Determination",
            Self::ExcitementSeeking => "Excitement-Seeking",
            Self::Intellect => "Intellect",
            Self::AttentionSeeking => "Attention-Seeking",
            Self::Cheerfulness => "Cheerfulness",
            Self::Liberalism => "Liberalism",
            Self::ArtisticInterests => "Artistic Interests",
            Self::Empathy => "Empathy",
            Self::WorkEthic => "Work Ethic",
            Self::Cautiousness => "Cautiousness",
            Self::Manipulativeness => "Manipulativeness",
            Self::Humility => "Humility",
            Self::Introspection => "Introspection",
            Self::Honesty => "Honesty",
            Self::Immoderation => "Immoderation",
            Self::SelfDiscipline => "Self-Discipline",
            Self::Recklessness => "Recklessness",
            Self::Calmness => "Calmness",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::Anxiety => {
                "Tendency to feel nervous, fearful, or overwhelmed under pressure (merged Anxiety & Vulnerability)."
            }
            Self::Gregariousness => {
                "Enthusiasm for making friends, socializing, and welcoming company (merged with Friendliness)."
            }
            Self::Trust => {
                "Belief in human goodness, sincerity, and the positive intentions of others."
            }
            Self::SelfEfficacy => {
                "Confidence in one's competence and ability to accomplish tasks successfully."
            }
            Self::Anger => "Tendency to experience irritation, quick temper, and anger.",
            Self::Fairness => {
                "Adherence to ethical rules, honesty in civic duty, and avoiding cheating."
            }
            Self::Orderliness => {
                "Preference for neatness, organization, structure, and avoiding mistakes."
            }
            Self::Dominance => {
                "Assertive, confrontational social orientation and willingness to take charge or challenge others."
            }
            Self::Emotionality => {
                "Tendency to experience emotions intensely and deeply (migrated from Openness to Neuroticism)."
            }
            Self::Adventurousness => {
                "Eagerness for variety, new experiences, and diverse interests over routine."
            }
            Self::Determination => {
                "Focused goal pursuit, resolve, and turning ambitious plans into decisive action."
            }
            Self::ExcitementSeeking => {
                "Craving high stimulation, fast-paced thrills, and novel adventures."
            }
            Self::Intellect => {
                "Enjoyment of solving complex intellectual problems and expanding vocabulary."
            }
            Self::AttentionSeeking => {
                "Preference regarding being the center of attention and discussing oneself (keyed toward modesty/reserve)."
            }
            Self::Cheerfulness => {
                "Disposition toward positive affect, joy, good spirits, and having fun."
            }
            Self::Liberalism => {
                "Openness to non-traditional values, political open-mindedness, and philosophical flexibility."
            }
            Self::ArtisticInterests => {
                "Appreciation and sensitivity for music, aesthetics, art, and natural beauty."
            }
            Self::Empathy => {
                "Compassionate concern for others' needs, feelings, and social causes."
            }
            Self::WorkEthic => {
                "Dedication to hard work, energetic diligence, and wholehearted commitment to tasks."
            }
            Self::Cautiousness => {
                "Careful forethought and deliberation before speaking or acting (reverse-keyed for impulsivity)."
            }
            Self::Manipulativeness => {
                "Tendency to use flattery, deception, or others for personal advantage (reverse-keyed for integrity)."
            }
            Self::Humility => {
                "Viewing oneself as an average person and avoiding looking down on others."
            }
            Self::Introspection => {
                "Tendency to reflect on internal thoughts, personal feelings, and fantasies."
            }
            Self::Honesty => {
                "Commitment to keeping promises, listening to conscience, and truthfulness."
            }
            Self::Immoderation => {
                "Difficulty resisting urges, temptations, or excessive indulgence."
            }
            Self::SelfDiscipline => {
                "Capacity to begin tasks promptly and persevere to completion without procrastination."
            }
            Self::Recklessness => {
                "Propensity for thrill-seeking, rash behavior, and acting wild or crazy."
            }
            Self::Calmness => "Preference for an unhurried, easygoing, and steady pace of life.",
        }
    }
}

#[inline]
fn matches_ci(input: &str, pattern: &str) -> bool {
    let input_bytes = input.as_bytes();
    let pattern_bytes = pattern.as_bytes();
    if input_bytes.len() < pattern_bytes.len() {
        return false;
    }
    input_bytes.windows(pattern_bytes.len()).any(|window| {
        window
            .iter()
            .zip(pattern_bytes)
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
    })
}

impl Facet {
    pub const ALL: [Facet; 28] = [
        Facet::Anxiety,
        Facet::Gregariousness,
        Facet::Trust,
        Facet::SelfEfficacy,
        Facet::Anger,
        Facet::Fairness,
        Facet::Orderliness,
        Facet::Dominance,
        Facet::Emotionality,
        Facet::Adventurousness,
        Facet::Determination,
        Facet::ExcitementSeeking,
        Facet::Intellect,
        Facet::AttentionSeeking,
        Facet::Cheerfulness,
        Facet::Liberalism,
        Facet::ArtisticInterests,
        Facet::Empathy,
        Facet::WorkEthic,
        Facet::Cautiousness,
        Facet::Manipulativeness,
        Facet::Humility,
        Facet::Introspection,
        Facet::Honesty,
        Facet::Immoderation,
        Facet::SelfDiscipline,
        Facet::Recklessness,
        Facet::Calmness,
    ];

    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if matches_ci(trimmed, "anxiety") {
            Some(Self::Anxiety)
        } else if matches_ci(trimmed, "gregariousness") {
            Some(Self::Gregariousness)
        } else if matches_ci(trimmed, "trust") {
            Some(Self::Trust)
        } else if matches_ci(trimmed, "self-efficacy")
            || matches_ci(trimmed, "self.efficacy")
            || matches_ci(trimmed, "selfefficacy")
        {
            Some(Self::SelfEfficacy)
        } else if matches_ci(trimmed, "anger") {
            Some(Self::Anger)
        } else if matches_ci(trimmed, "fairness") {
            Some(Self::Fairness)
        } else if matches_ci(trimmed, "orderliness") {
            Some(Self::Orderliness)
        } else if matches_ci(trimmed, "dominance") {
            Some(Self::Dominance)
        } else if matches_ci(trimmed, "emotionality") {
            Some(Self::Emotionality)
        } else if matches_ci(trimmed, "adventurousness") {
            Some(Self::Adventurousness)
        } else if matches_ci(trimmed, "determination") {
            Some(Self::Determination)
        } else if matches_ci(trimmed, "excitement-seeking")
            || matches_ci(trimmed, "excitement.seeking")
            || matches_ci(trimmed, "excitementseeking")
        {
            Some(Self::ExcitementSeeking)
        } else if matches_ci(trimmed, "intellect") {
            Some(Self::Intellect)
        } else if matches_ci(trimmed, "attention-seeking")
            || matches_ci(trimmed, "attention.seeking")
            || matches_ci(trimmed, "attentionseeking")
        {
            Some(Self::AttentionSeeking)
        } else if matches_ci(trimmed, "cheerfulness") {
            Some(Self::Cheerfulness)
        } else if matches_ci(trimmed, "liberalism") {
            Some(Self::Liberalism)
        } else if matches_ci(trimmed, "artistic interests")
            || matches_ci(trimmed, "artistic.interests")
            || matches_ci(trimmed, "artisticinterests")
        {
            Some(Self::ArtisticInterests)
        } else if matches_ci(trimmed, "empathy") {
            Some(Self::Empathy)
        } else if matches_ci(trimmed, "work ethic")
            || matches_ci(trimmed, "work.ethic")
            || matches_ci(trimmed, "workethic")
        {
            Some(Self::WorkEthic)
        } else if matches_ci(trimmed, "cautiousness") {
            Some(Self::Cautiousness)
        } else if matches_ci(trimmed, "manipulativeness") {
            Some(Self::Manipulativeness)
        } else if matches_ci(trimmed, "humility") {
            Some(Self::Humility)
        } else if matches_ci(trimmed, "introspection") {
            Some(Self::Introspection)
        } else if matches_ci(trimmed, "honesty") {
            Some(Self::Honesty)
        } else if matches_ci(trimmed, "immoderation") {
            Some(Self::Immoderation)
        } else if matches_ci(trimmed, "self-discipline")
            || matches_ci(trimmed, "self.discipline")
            || matches_ci(trimmed, "selfdiscipline")
        {
            Some(Self::SelfDiscipline)
        } else if matches_ci(trimmed, "recklessness") {
            Some(Self::Recklessness)
        } else if matches_ci(trimmed, "calmness") {
            Some(Self::Calmness)
        } else {
            None
        }
    }

    pub fn parent_trait(&self) -> Trait {
        match self {
            Self::Anxiety | Self::Anger | Self::Dominance | Self::Emotionality => {
                Trait::Neuroticism
            }
            Self::Determination
            | Self::WorkEthic
            | Self::SelfDiscipline
            | Self::Calmness
            | Self::SelfEfficacy
            | Self::Orderliness => Trait::Conscientiousness,
            Self::AttentionSeeking
            | Self::Cheerfulness
            | Self::Empathy
            | Self::Gregariousness
            | Self::Humility
            | Self::Trust => Trait::Sociability,
            Self::Adventurousness
            | Self::Intellect
            | Self::Liberalism
            | Self::ArtisticInterests
            | Self::Introspection => Trait::OpennessToExperience,
            Self::Manipulativeness | Self::Honesty | Self::Fairness => Trait::Integrity,
            Self::ExcitementSeeking
            | Self::Cautiousness
            | Self::Immoderation
            | Self::Recklessness => Trait::Impulsivity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Trait {
    Neuroticism,
    Sociability,
    Conscientiousness,
    Integrity,
    OpennessToExperience,
    Impulsivity,
}

impl Aspect for Trait {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Neuroticism => "Neuroticism",
            Self::Sociability => "Sociability",
            Self::Conscientiousness => "Conscientiousness",
            Self::Integrity => "Integrity",
            Self::OpennessToExperience => "Openness to Experience",
            Self::Impulsivity => "Impulsivity",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::Neuroticism => {
                "Tendency to experience negative emotions, stress, and interpersonal friction (Anger, Anxiety, Emotionality, Dominance)."
            }
            Self::Sociability => {
                "Broad social engagement blending extraverted affiliation with prosocial warmth (Gregariousness, Cheerfulness, Empathy, Trust, Attention-Seeking, Humility)."
            }
            Self::Conscientiousness => {
                "Self-discipline, diligence, organization, and deliberate goal pursuit (Self-Discipline, Work Ethic, Determination, Self-Efficacy, Orderliness, Calmness)."
            }
            Self::Integrity => {
                "Moral identity, adherence to ethical principles, and rejection of deceitful behavior (Fairness, Manipulativeness, Honesty)."
            }
            Self::OpennessToExperience => {
                "Cognitive exploration, intellectual curiosity, creativity, and aesthetic sensitivity (Intellect, Introspection, Artistic Interests, Adventurousness, Liberalism)."
            }
            Self::Impulsivity => {
                "Multifaceted behavioral regulation capturing thrill-seeking, rash action, and difficulty resisting impulses (Recklessness, Cautiousness, Excitement-Seeking, Immoderation)."
            }
        }
    }
}

impl Trait {
    pub const ALL: [Trait; 6] = [
        Trait::Neuroticism,
        Trait::Sociability,
        Trait::Conscientiousness,
        Trait::Integrity,
        Trait::OpennessToExperience,
        Trait::Impulsivity,
    ];

    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if matches_ci(trimmed, "neuroticism") {
            Some(Self::Neuroticism)
        } else if matches_ci(trimmed, "sociability") {
            Some(Self::Sociability)
        } else if matches_ci(trimmed, "conscientiousness") {
            Some(Self::Conscientiousness)
        } else if matches_ci(trimmed, "integrity") {
            Some(Self::Integrity)
        } else if matches_ci(trimmed, "openness") {
            Some(Self::OpennessToExperience)
        } else if matches_ci(trimmed, "impulsivity") {
            Some(Self::Impulsivity)
        } else {
            None
        }
    }

    pub fn parent_meta_trait(&self) -> MetaTrait {
        match self {
            Self::Neuroticism | Self::Conscientiousness => MetaTrait::Stability,
            Self::Sociability | Self::OpennessToExperience => MetaTrait::Plasticity,
            Self::Integrity | Self::Impulsivity => MetaTrait::Disinhibition,
        }
    }

    pub fn child_facets(&self) -> Vec<Facet> {
        Facet::ALL
            .iter()
            .copied()
            .filter(|f| f.parent_trait() == *self)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum MetaTrait {
    Stability,
    Plasticity,
    Disinhibition,
}

impl Aspect for MetaTrait {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Stability => "Stability",
            Self::Plasticity => "Plasticity",
            Self::Disinhibition => "Disinhibition",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::Stability => {
                "Reflects shared variance in emotional, motivational, and behavioral restraint (Neuroticism and Conscientiousness), capturing purposeful goal pursuit without emotional volatility."
            }
            Self::Plasticity => {
                "Reflects shared variance in exploration and engagement (Sociability and Openness to Experience), capturing tendencies toward exploring novel internal ideas and external social experiences."
            }
            Self::Disinhibition => {
                "A novel superordinate meta-trait combining Integrity and Impulsivity, spanning externalizing tendencies, ethical self-regulation, and behavioral control vs. rash action."
            }
        }
    }
}

impl MetaTrait {
    pub const ALL: [MetaTrait; 3] = [
        MetaTrait::Stability,
        MetaTrait::Plasticity,
        MetaTrait::Disinhibition,
    ];

    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if matches_ci(trimmed, "stability") {
            Some(Self::Stability)
        } else if matches_ci(trimmed, "plasticity") {
            Some(Self::Plasticity)
        } else if matches_ci(trimmed, "disinhibition") {
            Some(Self::Disinhibition)
        } else {
            None
        }
    }

    pub fn child_traits(&self) -> Vec<Trait> {
        Trait::ALL
            .iter()
            .copied()
            .filter(|t| t.parent_meta_trait() == *self)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreTier {
    VeryLow,
    Low,
    Average,
    High,
    VeryHigh,
}

impl ScoreTier {
    pub fn from_score(score: f32) -> Self {
        if score < -0.6 {
            Self::VeryLow
        } else if score < -0.2 {
            Self::Low
        } else if score <= 0.2 {
            Self::Average
        } else if score <= 0.6 {
            Self::High
        } else {
            Self::VeryHigh
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::VeryLow => "Very Low",
            Self::Low => "Low",
            Self::Average => "Average",
            Self::High => "High",
            Self::VeryHigh => "Very High",
        }
    }
}

/// Mean and standard deviation reference parameters on the normalized [-1.0, 1.0] scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NormParameters {
    pub mean: f32,
    pub sd: f32,
}

impl NormParameters {
    /// Creates custom normative parameters.
    pub const fn new(mean: f32, sd: f32) -> Self {
        Self { mean, sd }
    }

    /// Default theoretical prior: mean 0.0, standard deviation 0.40 on [-1.0, 1.0].
    pub const THEORETICAL_DEFAULT: Self = Self {
        mean: 0.0,
        sd: 0.40,
    };
}

/// Complete set of population norming parameters for Meta-Traits, Traits, and Facets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormSet {
    pub name: String,
    pub description: String,
    pub meta_traits: HashMap<MetaTrait, NormParameters>,
    pub traits: HashMap<Trait, NormParameters>,
    pub facets: HashMap<Facet, NormParameters>,
}

impl Default for NormSet {
    fn default() -> Self {
        Self::theoretical_tga()
    }
}

impl NormSet {
    /// Builds a uniform norming set applying the same [`NormParameters`] across every construct.
    pub fn with_uniform_parameters(
        name: impl Into<String>,
        description: impl Into<String>,
        params: NormParameters,
    ) -> Self {
        let meta_traits = MetaTrait::ALL
            .iter()
            .copied()
            .map(|m| (m, params))
            .collect();
        let traits = Trait::ALL.iter().copied().map(|t| (t, params)).collect();
        let facets = Facet::ALL.iter().copied().map(|f| (f, params)).collect();
        Self {
            name: name.into(),
            description: description.into(),
            meta_traits,
            traits,
            facets,
        }
    }

    /// Synthetic theoretical prior (`mean = 0.0`, `sd = 0.40`) pending empirical sample norms.
    pub fn theoretical_tga() -> Self {
        Self::with_uniform_parameters(
            "Theoretical TGA Prior (Zero-Centered)",
            "Uncalibrated theoretical Gaussian prior (mean = 0.0, sd = 0.40) on [-1.0, 1.0]; substitute empirical sample norms when available.",
            NormParameters::THEORETICAL_DEFAULT,
        )
    }

    /// Returns norming parameters for a meta-trait, falling back to `THEORETICAL_DEFAULT`.
    pub fn for_meta_trait(&self, meta: MetaTrait) -> NormParameters {
        self.meta_traits
            .get(&meta)
            .copied()
            .unwrap_or(NormParameters::THEORETICAL_DEFAULT)
    }

    /// Returns norming parameters for a trait, falling back to `THEORETICAL_DEFAULT`.
    pub fn for_trait(&self, r#trait: Trait) -> NormParameters {
        self.traits
            .get(&r#trait)
            .copied()
            .unwrap_or(NormParameters::THEORETICAL_DEFAULT)
    }

    /// Returns norming parameters for a facet, falling back to `THEORETICAL_DEFAULT`.
    pub fn for_facet(&self, facet: Facet) -> NormParameters {
        self.facets
            .get(&facet)
            .copied()
            .unwrap_or(NormParameters::THEORETICAL_DEFAULT)
    }
}

/// Active administration protocol controlling how many items are queued.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AdministrationMode {
    /// Full 221-item battery across all 28 facets.
    #[default]
    Full221,
    /// Rapid 112-item facet-balanced short form covering all 28 facets.
    Rapid112,
}

impl AdministrationMode {
    /// Full human-readable label for dialogs and selectors.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Full221 => "Full Battery (221 items)",
            Self::Rapid112 => "Rapid Protocol (112 items)",
        }
    }

    /// Concise badge/menu label.
    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Full221 => "Full (221)",
            Self::Rapid112 => "Rapid (112)",
        }
    }
}

/// Computes the deterministic 112-item facet-balanced subset indices (up to 4 items per
/// facet across all 28 facets, then topped up to 112 in `Optimized_Keys.csv` order).
pub fn rapid_112_indices(questions: &[Question]) -> Vec<usize> {
    const TARGET_LEN: usize = 112;
    let target = TARGET_LEN.min(questions.len());
    if questions.len() <= target {
        return (0..questions.len()).collect();
    }

    let mut selected = vec![false; questions.len()];
    let mut facet_counts: HashMap<Facet, usize> = HashMap::new();
    let mut selected_count = 0usize;

    // Pass 1: select up to 4 items per facet in optimized order so every facet is represented.
    for (idx, q) in questions.iter().enumerate() {
        if selected_count >= target {
            break;
        }
        let count = facet_counts.entry(q.facet.category).or_insert(0);
        if *count < 4 {
            selected[idx] = true;
            *count += 1;
            selected_count += 1;
        }
    }

    // Pass 2: fill any remaining slots up to 112 in optimized order.
    if selected_count < target {
        for (idx, flag) in selected.iter_mut().enumerate() {
            if selected_count >= target {
                break;
            }
            if !*flag {
                *flag = true;
                selected_count += 1;
            }
            let _ = idx;
        }
    }

    selected
        .into_iter()
        .enumerate()
        .filter_map(|(idx, keep)| if keep { Some(idx) } else { None })
        .collect()
}

/// Generates a plain-language 2-sentence interpretation for a construct score tier.
pub fn construct_narrative(name: &str, tier: ScoreTier, description: &str) -> String {
    let stance = match tier {
        ScoreTier::VeryLow => "strongly below the theoretical midpoint",
        ScoreTier::Low => "moderately below the theoretical midpoint",
        ScoreTier::Average => "near the balanced midpoint",
        ScoreTier::High => "moderately above the theoretical midpoint",
        ScoreTier::VeryHigh => "strongly above the theoretical midpoint",
    };
    let guidance = match tier {
        ScoreTier::VeryLow => format!(
            "You strongly lean toward the opposite pole of {}.",
            name.to_lowercase()
        ),
        ScoreTier::Low => format!(
            "You tend to express relatively restrained or opposite tendencies around {}.",
            name.to_lowercase()
        ),
        ScoreTier::Average => format!(
            "You generally show situational flexibility rather than a strong extreme on {}.",
            name.to_lowercase()
        ),
        ScoreTier::High => format!(
            "This dimension tends to be a noticeable part of your everyday style ({}).",
            description.trim_end_matches('.')
        ),
        ScoreTier::VeryHigh => format!(
            "This dimension is a prominent, consistent theme in your self-ratings ({}).",
            description.trim_end_matches('.')
        ),
    };
    format!(
        "Your {} score is {} ({} tier; Construct focus: {}). {}",
        name,
        stance,
        tier.label(),
        description.trim_end_matches('.'),
        guidance
    )
}

/// Abramowitz & Stegun 7.1.26 erf approximation for Gaussian CDF percentiles.
fn erf_approx(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let ax = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * ax);
    let y = 1.0
        - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t
            + 0.254829592)
            * t
            * (-ax * ax).exp();
    sign * y
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScoreWeight<T: Aspect> {
    pub category: T,
    pub weight: f32,
}

impl<T: Aspect> ScoreWeight<T> {
    pub fn new(category: T, weight: f32) -> Self {
        Self { category, weight }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Question {
    pub id: usize,
    pub label: String,
    pub text: String,
    pub facet: ScoreWeight<Facet>,
    pub r#trait: ScoreWeight<Trait>,
    pub meta_trait: ScoreWeight<MetaTrait>,
    pub response: Option<Response>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ScoreAccumulator {
    pub raw_score: f32,
    pub total_abs_weight: f32,
    pub total_sq_weight: f32,
    pub max_possible_abs_weight: f32,
    pub max_possible_sq_weight: f32,
    pub answered_count: usize,
    pub total_items: usize,
}

impl ScoreAccumulator {
    pub fn add_item_capacity(&mut self, weight: f32) {
        self.max_possible_abs_weight += weight.abs();
        self.max_possible_sq_weight += weight * weight;
        self.total_items += 1;
    }

    pub fn record_response(&mut self, score: f32, weight: f32) {
        self.raw_score += score * weight;
        self.total_abs_weight += weight.abs();
        self.total_sq_weight += weight * weight;
        self.answered_count += 1;
    }

    pub fn remove_response(&mut self, score: f32, weight: f32) {
        self.raw_score -= score * weight;
        self.total_abs_weight -= weight.abs();
        self.total_sq_weight -= weight * weight;
        self.answered_count = self.answered_count.saturating_sub(1);
    }

    pub fn normalized_score(&self) -> Option<f32> {
        if self.total_abs_weight == 0.0 {
            None
        } else {
            Some(self.raw_score / self.total_abs_weight)
        }
    }

    /// Standard Error (SE) projected onto the normalized [-1.0, 1.0] interval:
    /// SE = \frac{\sqrt{\sum w_i^2}}{\sum |w_i|} \times \sigma_{\text{scale}}
    /// where \sigma_{\text{scale}} = 0.5 (standard deviation scale of discrete Likert responses mapped to [-1, 1]).
    pub fn standard_error(&self) -> Option<f32> {
        if self.total_abs_weight == 0.0 {
            None
        } else {
            let response_sigma = 0.5_f32;
            let weighted_se = self.total_sq_weight.sqrt() / self.total_abs_weight;
            Some(weighted_se * response_sigma)
        }
    }

    pub fn tier(&self) -> Option<ScoreTier> {
        self.normalized_score().map(ScoreTier::from_score)
    }

    /// Computes standardized z-score relative to [`NormParameters`] (`None` if uncomputed or `sd <= 0`).
    pub fn z_score(&self, norms: NormParameters) -> Option<f32> {
        if !norms.sd.is_finite() || norms.sd <= f32::EPSILON || !norms.mean.is_finite() {
            return None;
        }
        self.normalized_score().map(|s| (s - norms.mean) / norms.sd)
    }

    /// Computes estimated percentile in `[0.0, 100.0]` under a Gaussian norming assumption.
    pub fn percentile_estimate(&self, norms: NormParameters) -> Option<f32> {
        self.z_score(norms).map(|z| {
            let cdf = 0.5 * (1.0 + erf_approx(f64::from(z) / std::f64::consts::SQRT_2));
            (cdf * 100.0).clamp(0.0, 100.0) as f32
        })
    }

    /// Generates a plain-language interpretation string for the given construct.
    pub fn narrative(&self, aspect: &impl Aspect) -> Option<String> {
        self.tier()
            .map(|tier| construct_narrative(aspect.display_name(), tier, aspect.description()))
    }
}

/// Action entry in the undo/redo history stack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HistoryAction {
    /// Single question answer change or clear.
    AnswerChange {
        question_idx: usize,
        old_response: Option<Response>,
        new_response: Option<Response>,
        old_focus_idx: usize,
        new_focus_idx: usize,
    },
    /// Bulk state change (e.g. shared link loaded, reset, import, or batch compaction).
    StateSnapshot {
        old_responses: Vec<Option<Response>>,
        new_responses: Vec<Option<Response>>,
        old_show_results: bool,
        new_show_results: bool,
        old_focus_idx: usize,
        new_focus_idx: usize,
        label: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct QuestionnaireState {
    pub questions: Vec<Question>,
    /// Queue of question indices (into `questions`) remaining to be answered.
    pub pending_queue: VecDeque<usize>,
    /// Current question index being displayed in focus mode.
    pub current_focus_idx: usize,
    /// Whether results are explicitly revealed or completed.
    pub show_results: bool,
    /// Detailed statistics toggle in results view.
    pub show_detailed_stats: bool,
    /// Cached score accumulators.
    #[serde(skip)]
    pub facet_acc: HashMap<Facet, ScoreAccumulator>,
    #[serde(skip)]
    pub trait_acc: HashMap<Trait, ScoreAccumulator>,
    #[serde(skip)]
    pub meta_trait_acc: HashMap<MetaTrait, ScoreAccumulator>,
    /// Undo history stack storing actions.
    #[serde(default)]
    pub undo_stack: Vec<HistoryAction>,
    /// Redo history stack storing undone actions.
    #[serde(default)]
    pub redo_stack: Vec<HistoryAction>,
    /// Active administration protocol (`Full221` vs. `Rapid112`).
    #[serde(default)]
    pub admin_mode: AdministrationMode,
}

impl Default for QuestionnaireState {
    fn default() -> Self {
        Self::from_embedded_data()
    }
}

impl QuestionnaireState {
    /// Loads all questions from the embedded datasets in optimized order.
    pub fn from_embedded_data() -> Self {
        let questions = load_optimized_questions();
        let queue: VecDeque<usize> = (0..questions.len()).collect();
        let mut state = Self {
            questions,
            pending_queue: queue,
            current_focus_idx: 0,
            show_results: false,
            show_detailed_stats: false,
            facet_acc: HashMap::new(),
            trait_acc: HashMap::new(),
            meta_trait_acc: HashMap::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            admin_mode: AdministrationMode::Full221,
        };
        state.rebuild_cache();
        state
    }

    /// Returns the ordered list of active question indices for the current [`AdministrationMode`].
    pub fn active_question_indices(&self) -> Vec<usize> {
        match self.admin_mode {
            AdministrationMode::Full221 => (0..self.questions.len()).collect(),
            AdministrationMode::Rapid112 => rapid_112_indices(&self.questions),
        }
    }

    /// Returns a boolean mask indexed by `self.questions` marking active protocol items.
    pub fn active_question_mask(&self) -> Vec<bool> {
        let mut mask = vec![false; self.questions.len()];
        for idx in self.active_question_indices() {
            if idx < mask.len() {
                mask[idx] = true;
            }
        }
        mask
    }

    /// Switches between [`AdministrationMode::Full221`] and [`AdministrationMode::Rapid112`]
    /// while preserving every previously recorded response.
    pub fn set_administration_mode(&mut self, mode: AdministrationMode) {
        if self.admin_mode == mode {
            return;
        }
        self.admin_mode = mode;
        self.rebuild_cache();
        if let Some(&first_pending) = self.pending_queue.front() {
            let mask = self.active_question_mask();
            let focus_invalid = self.questions.get(self.current_focus_idx).is_none_or(|q| {
                !mask.get(self.current_focus_idx).copied().unwrap_or(false) || q.response.is_some()
            });
            if focus_invalid {
                self.current_focus_idx = first_pending;
            }
        }
    }

    /// Captures compact construct score vectors for longitudinal snapshots.
    pub fn snapshot_construct_scores(
        &self,
    ) -> (
        Vec<crate::SnapshotConstructScore>,
        Vec<crate::SnapshotConstructScore>,
        Vec<crate::SnapshotConstructScore>,
    ) {
        let meta = MetaTrait::ALL
            .iter()
            .enumerate()
            .map(|(i, &m)| {
                let acc = self.meta_trait_acc.get(&m).copied().unwrap_or_default();
                crate::SnapshotConstructScore {
                    id: (i + 1) as u8,
                    name: m.display_name().to_string(),
                    normalized_score: acc.normalized_score(),
                    standard_error: acc.standard_error(),
                    tier_label: acc.tier().map(|t| t.label().to_string()),
                }
            })
            .collect();
        let traits = Trait::ALL
            .iter()
            .enumerate()
            .map(|(i, &t)| {
                let acc = self.trait_acc.get(&t).copied().unwrap_or_default();
                crate::SnapshotConstructScore {
                    id: (i + 1) as u8,
                    name: t.display_name().to_string(),
                    normalized_score: acc.normalized_score(),
                    standard_error: acc.standard_error(),
                    tier_label: acc.tier().map(|tier| tier.label().to_string()),
                }
            })
            .collect();
        let facets = Facet::ALL
            .iter()
            .enumerate()
            .map(|(i, &f)| {
                let acc = self.facet_acc.get(&f).copied().unwrap_or_default();
                crate::SnapshotConstructScore {
                    id: (i + 1) as u8,
                    name: f.display_name().to_string(),
                    normalized_score: acc.normalized_score(),
                    standard_error: acc.standard_error(),
                    tier_label: acc.tier().map(|tier| tier.label().to_string()),
                }
            })
            .collect();
        (meta, traits, facets)
    }

    /// Fully rebuilds accumulators and the protocol-filtered pending queue from the question list.
    pub fn rebuild_cache(&mut self) {
        self.facet_acc.clear();
        self.trait_acc.clear();
        self.meta_trait_acc.clear();

        // 1. Initialize capacities for all facets, traits, meta-traits
        for q in &self.questions {
            self.facet_acc
                .entry(q.facet.category)
                .or_default()
                .add_item_capacity(q.facet.weight);
            self.trait_acc
                .entry(q.r#trait.category)
                .or_default()
                .add_item_capacity(q.r#trait.weight);
            self.meta_trait_acc
                .entry(q.meta_trait.category)
                .or_default()
                .add_item_capacity(q.meta_trait.weight);
        }

        // 2. Re-apply recorded responses
        for q in &self.questions {
            if let Some(resp) = q.response {
                let score = resp.to_score();
                if let Some(acc) = self.facet_acc.get_mut(&q.facet.category) {
                    acc.record_response(score, q.facet.weight);
                }
                if let Some(acc) = self.trait_acc.get_mut(&q.r#trait.category) {
                    acc.record_response(score, q.r#trait.weight);
                }
                if let Some(acc) = self.meta_trait_acc.get_mut(&q.meta_trait.category) {
                    acc.record_response(score, q.meta_trait.weight);
                }
            }
        }

        // 3. Rebuild pending queue from unanswered active-protocol questions
        let mask = self.active_question_mask();
        let active_indices = self.active_question_indices();
        let unanswered: Vec<usize> = active_indices
            .into_iter()
            .filter(|&i| self.questions.get(i).is_some_and(|q| q.response.is_none()))
            .collect();

        // Keep current queue order where possible, retaining only active unanswered indices
        let mut new_queue = VecDeque::new();
        for &idx in &self.pending_queue {
            if idx < self.questions.len()
                && mask.get(idx).copied().unwrap_or(false)
                && self.questions[idx].response.is_none()
                && !new_queue.contains(&idx)
            {
                new_queue.push_back(idx);
            }
        }
        for idx in unanswered {
            if !new_queue.contains(&idx) {
                new_queue.push_back(idx);
            }
        }
        self.pending_queue = new_queue;

        // If all active protocol questions are answered, auto-enable show_results
        if self.pending_queue.is_empty() && !self.questions.is_empty() {
            self.show_results = true;
        }

        if let Some(&first_pending) = self.pending_queue.front()
            && (self.questions.get(self.current_focus_idx).is_none()
                || !mask.get(self.current_focus_idx).copied().unwrap_or(false)
                || self.questions[self.current_focus_idx].response.is_some())
        {
            self.current_focus_idx = first_pending;
        }
    }

    /// Whether an undo action is currently available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Whether a redo action is currently available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Returns a snapshot of all current question responses.
    pub fn current_responses_snapshot(&self) -> Vec<Option<Response>> {
        self.questions.iter().map(|q| q.response).collect()
    }

    /// Pushes an action to the undo stack and clears the redo stack to begin a new branch.
    pub fn push_action(&mut self, action: HistoryAction) {
        self.undo_stack.push(action);
        self.redo_stack.clear();

        // Auto-compact history if it grows excessively large (> 120 entries)
        if self.undo_stack.len() > 120 {
            self.compact_history(80);
        }
    }

    /// Sets response for a given question index and updates score accumulators.
    pub fn answer_question(&mut self, question_idx: usize, response: Response) -> bool {
        if question_idx >= self.questions.len() {
            return false;
        }

        let old_response = self.questions[question_idx].response;
        let old_focus = self.current_focus_idx;

        let q = &mut self.questions[question_idx];
        q.response = Some(response);

        // Update accumulators
        let score = response.to_score();
        if let Some(old_r) = old_response {
            let old_score = old_r.to_score();
            if let Some(acc) = self.facet_acc.get_mut(&q.facet.category) {
                acc.remove_response(old_score, q.facet.weight);
                acc.record_response(score, q.facet.weight);
            }
            if let Some(acc) = self.trait_acc.get_mut(&q.r#trait.category) {
                acc.remove_response(old_score, q.r#trait.weight);
                acc.record_response(score, q.r#trait.weight);
            }
            if let Some(acc) = self.meta_trait_acc.get_mut(&q.meta_trait.category) {
                acc.remove_response(old_score, q.meta_trait.weight);
                acc.record_response(score, q.meta_trait.weight);
            }
        } else {
            if let Some(acc) = self.facet_acc.get_mut(&q.facet.category) {
                acc.record_response(score, q.facet.weight);
            }
            if let Some(acc) = self.trait_acc.get_mut(&q.r#trait.category) {
                acc.record_response(score, q.r#trait.weight);
            }
            if let Some(acc) = self.meta_trait_acc.get_mut(&q.meta_trait.category) {
                acc.record_response(score, q.meta_trait.weight);
            }
        }

        // Remove from pending queue if present
        self.pending_queue.retain(|&idx| idx != question_idx);

        // Auto-advance to the next pending question if any remain
        if let Some(&next_idx) = self.pending_queue.front() {
            self.current_focus_idx = next_idx;
        } else {
            // All answered! Auto-reveal results
            self.show_results = true;
        }

        let new_focus = self.current_focus_idx;
        self.push_action(HistoryAction::AnswerChange {
            question_idx,
            old_response,
            new_response: Some(response),
            old_focus_idx: old_focus,
            new_focus_idx: new_focus,
        });

        true
    }

    /// Clears response for a question.
    pub fn clear_response(&mut self, question_idx: usize) -> bool {
        if question_idx >= self.questions.len() {
            return false;
        }

        let old_response = self.questions[question_idx].response;
        if old_response.is_none() {
            return false;
        }

        let old_focus = self.current_focus_idx;
        let q = &mut self.questions[question_idx];
        if let Some(old_r) = q.response.take() {
            let old_score = old_r.to_score();
            if let Some(acc) = self.facet_acc.get_mut(&q.facet.category) {
                acc.remove_response(old_score, q.facet.weight);
            }
            if let Some(acc) = self.trait_acc.get_mut(&q.r#trait.category) {
                acc.remove_response(old_score, q.r#trait.weight);
            }
            if let Some(acc) = self.meta_trait_acc.get_mut(&q.meta_trait.category) {
                acc.remove_response(old_score, q.meta_trait.weight);
            }

            if !self.pending_queue.contains(&question_idx) {
                self.pending_queue.push_front(question_idx);
            }

            self.push_action(HistoryAction::AnswerChange {
                question_idx,
                old_response,
                new_response: None,
                old_focus_idx: old_focus,
                new_focus_idx: self.current_focus_idx,
            });

            true
        } else {
            false
        }
    }

    /// Loads a full response snapshot with complete undo/redo support.
    pub fn load_snapshot_with_undo(
        &mut self,
        new_responses: Vec<Option<Response>>,
        new_show_results: bool,
        label: &str,
    ) {
        let old_responses = self.current_responses_snapshot();
        let old_show_results = self.show_results;
        let old_focus = self.current_focus_idx;

        for (i, &resp) in new_responses.iter().enumerate() {
            if i < self.questions.len() {
                self.questions[i].response = resp;
            }
        }
        self.show_results = new_show_results;
        self.rebuild_cache();

        let new_focus = self.current_focus_idx;
        let new_responses_cloned = self.current_responses_snapshot();

        self.push_action(HistoryAction::StateSnapshot {
            old_responses,
            new_responses: new_responses_cloned,
            old_show_results,
            new_show_results,
            old_focus_idx: old_focus,
            new_focus_idx: new_focus,
            label: label.to_string(),
        });
    }

    /// Resets all answers with complete undo capability.
    pub fn reset_with_undo(&mut self) {
        let empty = vec![None; self.questions.len()];
        self.load_snapshot_with_undo(empty, false, "Reset Assessment");
    }

    /// Alias for reset_with_undo.
    pub fn reset(&mut self) {
        self.reset_with_undo();
    }

    /// Reverts the most recent operation and places it on the redo stack.
    pub fn undo(&mut self) -> bool {
        if let Some(action) = self.undo_stack.pop() {
            match &action {
                HistoryAction::AnswerChange {
                    question_idx,
                    old_response,
                    old_focus_idx,
                    ..
                } => {
                    let idx = *question_idx;
                    if idx < self.questions.len() {
                        let current_r = self.questions[idx].response;
                        if let Some(r) = current_r {
                            let score = r.to_score();
                            let q = &self.questions[idx];
                            if let Some(acc) = self.facet_acc.get_mut(&q.facet.category) {
                                acc.remove_response(score, q.facet.weight);
                            }
                            if let Some(acc) = self.trait_acc.get_mut(&q.r#trait.category) {
                                acc.remove_response(score, q.r#trait.weight);
                            }
                            if let Some(acc) = self.meta_trait_acc.get_mut(&q.meta_trait.category) {
                                acc.remove_response(score, q.meta_trait.weight);
                            }
                        }

                        self.questions[idx].response = *old_response;
                        if let Some(prev) = *old_response {
                            let score = prev.to_score();
                            let q = &self.questions[idx];
                            if let Some(acc) = self.facet_acc.get_mut(&q.facet.category) {
                                acc.record_response(score, q.facet.weight);
                            }
                            if let Some(acc) = self.trait_acc.get_mut(&q.r#trait.category) {
                                acc.record_response(score, q.r#trait.weight);
                            }
                            if let Some(acc) = self.meta_trait_acc.get_mut(&q.meta_trait.category) {
                                acc.record_response(score, q.meta_trait.weight);
                            }
                            self.pending_queue.retain(|&i| i != idx);
                        } else if !self.pending_queue.contains(&idx) {
                            self.pending_queue.push_front(idx);
                        }

                        self.current_focus_idx = *old_focus_idx;
                    }
                }
                HistoryAction::StateSnapshot {
                    old_responses,
                    old_show_results,
                    old_focus_idx,
                    ..
                } => {
                    for (i, &resp) in old_responses.iter().enumerate() {
                        if i < self.questions.len() {
                            self.questions[i].response = resp;
                        }
                    }
                    self.show_results = *old_show_results;
                    self.current_focus_idx = *old_focus_idx;
                    self.rebuild_cache();
                }
            }

            self.redo_stack.push(action);
            return true;
        }
        false
    }

    /// Redoes the most recently undone operation.
    pub fn redo(&mut self) -> bool {
        if let Some(action) = self.redo_stack.pop() {
            match &action {
                HistoryAction::AnswerChange {
                    question_idx,
                    new_response,
                    new_focus_idx,
                    ..
                } => {
                    let idx = *question_idx;
                    if idx < self.questions.len() {
                        let current_r = self.questions[idx].response;
                        if let Some(r) = current_r {
                            let score = r.to_score();
                            let q = &self.questions[idx];
                            if let Some(acc) = self.facet_acc.get_mut(&q.facet.category) {
                                acc.remove_response(score, q.facet.weight);
                            }
                            if let Some(acc) = self.trait_acc.get_mut(&q.r#trait.category) {
                                acc.remove_response(score, q.r#trait.weight);
                            }
                            if let Some(acc) = self.meta_trait_acc.get_mut(&q.meta_trait.category) {
                                acc.remove_response(score, q.meta_trait.weight);
                            }
                        }

                        self.questions[idx].response = *new_response;
                        if let Some(nxt) = *new_response {
                            let score = nxt.to_score();
                            let q = &self.questions[idx];
                            if let Some(acc) = self.facet_acc.get_mut(&q.facet.category) {
                                acc.record_response(score, q.facet.weight);
                            }
                            if let Some(acc) = self.trait_acc.get_mut(&q.r#trait.category) {
                                acc.record_response(score, q.r#trait.weight);
                            }
                            if let Some(acc) = self.meta_trait_acc.get_mut(&q.meta_trait.category) {
                                acc.record_response(score, q.meta_trait.weight);
                            }
                            self.pending_queue.retain(|&i| i != idx);
                        } else if !self.pending_queue.contains(&idx) {
                            self.pending_queue.push_front(idx);
                        }

                        self.current_focus_idx = *new_focus_idx;
                    }
                }
                HistoryAction::StateSnapshot {
                    new_responses,
                    new_show_results,
                    new_focus_idx,
                    ..
                } => {
                    for (i, &resp) in new_responses.iter().enumerate() {
                        if i < self.questions.len() {
                            self.questions[i].response = resp;
                        }
                    }
                    self.show_results = *new_show_results;
                    self.current_focus_idx = *new_focus_idx;
                    self.rebuild_cache();
                }
            }

            self.undo_stack.push(action);
            return true;
        }
        false
    }

    /// Compacts undo history so that the density of history is relative to recency.
    /// Recent steps remain 1:1, whereas older steps are consolidated into larger composite checkpoints.
    pub fn compact_history(&mut self, target_max_entries: usize) {
        let len = self.undo_stack.len();
        if len <= target_max_entries || len < 4 {
            return;
        }

        let target = target_max_entries.max(4);
        let tier0_limit = (target * 35 / 100).max(2);
        let tier1_limit = target;
        let tier2_limit = target * 5 / 2;

        let mut compacted = Vec::new();
        let total_questions = self.questions.len();

        let mut idx = 0;
        while idx < len {
            let distance_from_top = len - 1 - idx;
            let group_size = if distance_from_top < tier0_limit {
                1
            } else if distance_from_top < tier1_limit {
                2
            } else if distance_from_top < tier2_limit {
                4
            } else {
                8
            };

            let end = (idx + group_size).min(len);
            let chunk = &self.undo_stack[idx..end];

            if chunk.len() == 1 {
                compacted.push(chunk[0].clone());
            } else {
                // Consolidate chunk into a single StateSnapshot
                let first = &chunk[0];
                let last = &chunk[chunk.len() - 1];

                let (old_focus_idx, old_show_results) = match first {
                    HistoryAction::AnswerChange { old_focus_idx, .. } => (*old_focus_idx, false),
                    HistoryAction::StateSnapshot {
                        old_focus_idx,
                        old_show_results,
                        ..
                    } => (*old_focus_idx, *old_show_results),
                };

                let (new_focus_idx, new_show_results) = match last {
                    HistoryAction::AnswerChange { new_focus_idx, .. } => (*new_focus_idx, false),
                    HistoryAction::StateSnapshot {
                        new_focus_idx,
                        new_show_results,
                        ..
                    } => (*new_focus_idx, *new_show_results),
                };

                // Track diff across chunk:
                let mut old_map: HashMap<usize, Option<Response>> = HashMap::new();
                let mut new_map: HashMap<usize, Option<Response>> = HashMap::new();

                for item in chunk {
                    match item {
                        HistoryAction::AnswerChange {
                            question_idx,
                            old_response,
                            new_response,
                            ..
                        } => {
                            old_map.entry(*question_idx).or_insert(*old_response);
                            new_map.insert(*question_idx, *new_response);
                        }
                        HistoryAction::StateSnapshot {
                            old_responses,
                            new_responses,
                            ..
                        } => {
                            for (q_i, &resp) in old_responses.iter().enumerate() {
                                old_map.entry(q_i).or_insert(resp);
                            }
                            for (q_i, &resp) in new_responses.iter().enumerate() {
                                new_map.insert(q_i, resp);
                            }
                        }
                    }
                }

                // If chunk was purely a single net question change, keep it as AnswerChange
                let changed_keys: Vec<usize> = old_map
                    .keys()
                    .copied()
                    .filter(|k| old_map.get(k) != new_map.get(k))
                    .collect();

                if changed_keys.len() == 1 {
                    let k = changed_keys[0];
                    compacted.push(HistoryAction::AnswerChange {
                        question_idx: k,
                        old_response: old_map.get(&k).copied().flatten(),
                        new_response: new_map.get(&k).copied().flatten(),
                        old_focus_idx,
                        new_focus_idx,
                    });
                } else {
                    let mut old_responses = vec![None; total_questions];
                    let mut new_responses = vec![None; total_questions];
                    for (k, v) in old_map {
                        if k < total_questions {
                            old_responses[k] = v;
                        }
                    }
                    for (k, v) in new_map {
                        if k < total_questions {
                            new_responses[k] = v;
                        }
                    }

                    compacted.push(HistoryAction::StateSnapshot {
                        old_responses,
                        new_responses,
                        old_show_results,
                        new_show_results,
                        old_focus_idx,
                        new_focus_idx,
                        label: format!("Consolidated History ({} steps)", chunk.len()),
                    });
                }
            }

            idx = end;
        }

        self.undo_stack = compacted;
    }

    /// Skips the currently focused question, deferring it to the back of the queue.
    pub fn skip_current(&mut self) {
        if self.pending_queue.is_empty() {
            // If all are answered, cycle to next index in total list
            if !self.questions.is_empty() {
                self.current_focus_idx = (self.current_focus_idx + 1) % self.questions.len();
            }
            return;
        }

        // If current focus is in the pending queue
        if let Some(pos) = self
            .pending_queue
            .iter()
            .position(|&idx| idx == self.current_focus_idx)
        {
            let popped = self.pending_queue.remove(pos).unwrap();
            self.pending_queue.push_back(popped);
            if let Some(&next_idx) = self.pending_queue.front() {
                self.current_focus_idx = next_idx;
            }
        } else {
            // If viewing an already answered item, jump to next pending or next item
            if let Some(&next_idx) = self.pending_queue.front() {
                self.current_focus_idx = next_idx;
            } else {
                self.current_focus_idx = (self.current_focus_idx + 1) % self.questions.len();
            }
        }
    }

    /// Navigates to the previous question.
    pub fn navigate_previous(&mut self) {
        if self.questions.is_empty() {
            return;
        }
        if self.current_focus_idx == 0 {
            self.current_focus_idx = self.questions.len() - 1;
        } else {
            self.current_focus_idx -= 1;
        }
    }

    /// Navigates to the next question.
    pub fn navigate_next(&mut self) {
        if self.questions.is_empty() {
            return;
        }
        self.current_focus_idx = (self.current_focus_idx + 1) % self.questions.len();
    }

    /// Navigates to the closest unanswered question looking backwards.
    pub fn navigate_previous_unanswered(&mut self) {
        if self.questions.is_empty() {
            return;
        }
        let len = self.questions.len();
        let start = self.current_focus_idx;
        for i in 1..=len {
            let idx = (start + len - i) % len;
            if self.questions[idx].response.is_none() {
                self.current_focus_idx = idx;
                break;
            }
        }
    }

    /// Navigates to the closest unanswered question looking forwards.
    pub fn navigate_next_unanswered(&mut self) {
        if self.questions.is_empty() {
            return;
        }
        let len = self.questions.len();
        let start = self.current_focus_idx;
        for i in 1..=len {
            let idx = (start + i) % len;
            if self.questions[idx].response.is_none() {
                self.current_focus_idx = idx;
                break;
            }
        }
    }

    /// Count of questions that have been answered.
    pub fn answered_count(&self) -> usize {
        self.questions
            .iter()
            .filter(|q| q.response.is_some())
            .count()
    }

    /// Count of questions that remain unanswered in the active protocol.
    pub fn unanswered_count(&self) -> usize {
        self.active_total_count()
            .saturating_sub(self.active_answered_count())
    }

    /// Total questions count across the underlying 221-item bank.
    pub fn total_questions(&self) -> usize {
        self.questions.len()
    }

    /// Total items belonging to the currently active [`AdministrationMode`] (`221` or `112`).
    pub fn active_total_count(&self) -> usize {
        match self.admin_mode {
            AdministrationMode::Full221 => self.questions.len(),
            AdministrationMode::Rapid112 => rapid_112_indices(&self.questions).len(),
        }
    }

    /// Count of answered items belonging to the currently active [`AdministrationMode`].
    pub fn active_answered_count(&self) -> usize {
        match self.admin_mode {
            AdministrationMode::Full221 => self.answered_count(),
            AdministrationMode::Rapid112 => rapid_112_indices(&self.questions)
                .into_iter()
                .filter(|&i| self.questions.get(i).is_some_and(|q| q.response.is_some()))
                .count(),
        }
    }

    /// Completion rate in [0.0, 1.0] for the active protocol.
    pub fn completion_rate(&self) -> f32 {
        let denom = self.active_total_count();
        if denom == 0 {
            0.0
        } else {
            self.active_answered_count() as f32 / denom as f32
        }
    }

    /// Checks if all items of the active protocol have been answered.
    pub fn is_completed(&self) -> bool {
        let denom = self.active_total_count();
        denom > 0 && self.active_answered_count() == denom
    }
}

type RawQuestionMap = HashMap<
    String,
    (
        String,
        ScoreWeight<Facet>,
        ScoreWeight<Trait>,
        ScoreWeight<MetaTrait>,
    ),
>;

static PARSED_QUESTIONS_CACHE: std::sync::OnceLock<Vec<Question>> = std::sync::OnceLock::new();

/// Parses the embedded CSV files and returns the questions ordered according to `Optimized_Keys.csv`.
/// Results are cached statically in memory via `OnceLock`.
pub fn load_optimized_questions() -> Vec<Question> {
    PARSED_QUESTIONS_CACHE
        .get_or_init(parse_and_order_raw_questions)
        .clone()
}

fn parse_and_order_raw_questions() -> Vec<Question> {
    // 1. Parse Distillined Key.csv into a map: Label -> Question
    let mut raw_map: RawQuestionMap = HashMap::new();

    for line in DISTILLED_KEY_CSV.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Handle CSV split with possible quoted columns
        let fields = parse_csv_line(line);
        if fields.len() < 8 {
            continue;
        }

        let label = fields[0].clone();
        let question_text = fields[1].clone();
        let facet_str = &fields[2];
        let trait_str = &fields[3];
        let meta_trait_str = &fields[4];

        let facet_w: f32 = fields[5].parse().unwrap_or(0.0);
        let trait_w: f32 = fields[6].parse().unwrap_or(0.0);
        let meta_trait_w: f32 = fields[7].parse().unwrap_or(0.0);

        let facet = Facet::parse(facet_str).expect("Valid facet name");
        let r#trait = Trait::parse(trait_str).expect("Valid trait name");
        let meta_trait = MetaTrait::parse(meta_trait_str).expect("Valid meta-trait name");

        raw_map.insert(
            label,
            (
                question_text,
                ScoreWeight::new(facet, facet_w),
                ScoreWeight::new(r#trait, trait_w),
                ScoreWeight::new(meta_trait, meta_trait_w),
            ),
        );
    }

    // 2. Read sequence of labels from Optimized_Keys.csv
    let opt_lines: Vec<&str> = OPTIMIZED_KEYS_CSV
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    let sequence_labels: Vec<String> = if opt_lines.len() >= 2 {
        // Second row contains the labels
        parse_csv_line(opt_lines[1])
    } else if opt_lines.len() == 1 {
        parse_csv_line(opt_lines[0])
    } else {
        vec![]
    };

    let mut ordered_questions = Vec::with_capacity(sequence_labels.len());
    for (idx, label) in sequence_labels.into_iter().enumerate() {
        if let Some((text, facet_w, trait_w, meta_trait_w)) = raw_map.remove(&label) {
            ordered_questions.push(Question {
                id: idx + 1,
                label,
                text,
                facet: facet_w,
                r#trait: trait_w,
                meta_trait: meta_trait_w,
                response: None,
            });
        }
    }

    ordered_questions
}

pub use spodeian_export::parse_csv_line;
