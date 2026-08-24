use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt;

/// Embed source CSV datasets at compile time.
pub const DISTILLED_KEY_CSV: &str = include_str!("../../../Distillined Key.csv");
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
            Self::SelfEfficacy => "Self-efficacy",
            Self::Anger => "Anger",
            Self::Fairness => "Fairness",
            Self::Orderliness => "Orderliness",
            Self::Dominance => "Dominance",
            Self::Emotionality => "Emotionality",
            Self::Adventurousness => "Adventurousness",
            Self::Determination => "Determination",
            Self::ExcitementSeeking => "Excitement-seeking",
            Self::Intellect => "Intellect",
            Self::AttentionSeeking => "Attention-seeking",
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
            Self::SelfDiscipline => "Self-discipline",
            Self::Recklessness => "Recklessness",
            Self::Calmness => "Calmness",
        }
    }
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
        let cleaned = s.trim().to_lowercase();
        if cleaned.contains("anxiety") {
            Some(Self::Anxiety)
        } else if cleaned.contains("gregariousness") {
            Some(Self::Gregariousness)
        } else if cleaned.contains("trust") {
            Some(Self::Trust)
        } else if cleaned.contains("self-efficacy") || cleaned.contains("self.efficacy") {
            Some(Self::SelfEfficacy)
        } else if cleaned.contains("anger") {
            Some(Self::Anger)
        } else if cleaned.contains("fairness") {
            Some(Self::Fairness)
        } else if cleaned.contains("orderliness") {
            Some(Self::Orderliness)
        } else if cleaned.contains("dominance") {
            Some(Self::Dominance)
        } else if cleaned.contains("emotionality") {
            Some(Self::Emotionality)
        } else if cleaned.contains("adventurousness") {
            Some(Self::Adventurousness)
        } else if cleaned.contains("determination") {
            Some(Self::Determination)
        } else if cleaned.contains("excitement-seeking") || cleaned.contains("excitement.seeking") {
            Some(Self::ExcitementSeeking)
        } else if cleaned.contains("intellect") {
            Some(Self::Intellect)
        } else if cleaned.contains("attention-seeking") || cleaned.contains("attention.seeking") {
            Some(Self::AttentionSeeking)
        } else if cleaned.contains("cheerfulness") {
            Some(Self::Cheerfulness)
        } else if cleaned.contains("liberalism") {
            Some(Self::Liberalism)
        } else if cleaned.contains("artistic interests") || cleaned.contains("artistic.interests") {
            Some(Self::ArtisticInterests)
        } else if cleaned.contains("empathy") {
            Some(Self::Empathy)
        } else if cleaned.contains("work ethic") || cleaned.contains("work.ethic") {
            Some(Self::WorkEthic)
        } else if cleaned.contains("cautiousness") {
            Some(Self::Cautiousness)
        } else if cleaned.contains("manipulativeness") {
            Some(Self::Manipulativeness)
        } else if cleaned.contains("humility") {
            Some(Self::Humility)
        } else if cleaned.contains("introspection") {
            Some(Self::Introspection)
        } else if cleaned.contains("honesty") {
            Some(Self::Honesty)
        } else if cleaned.contains("immoderation") {
            Some(Self::Immoderation)
        } else if cleaned.contains("self-discipline") || cleaned.contains("self.discipline") {
            Some(Self::SelfDiscipline)
        } else if cleaned.contains("recklessness") {
            Some(Self::Recklessness)
        } else if cleaned.contains("calmness") {
            Some(Self::Calmness)
        } else {
            None
        }
    }

    pub fn parent_trait(&self) -> Trait {
        match self {
            Self::Anxiety | Self::Anger | Self::Dominance | Self::Emotionality => Trait::Neuroticism,
            Self::Determination | Self::WorkEthic | Self::SelfDiscipline | Self::Calmness | Self::SelfEfficacy | Self::Orderliness => Trait::Conscientiousness,
            Self::AttentionSeeking | Self::Cheerfulness | Self::Empathy | Self::Gregariousness | Self::Humility | Self::Trust => Trait::Sociability,
            Self::Adventurousness | Self::Intellect | Self::Liberalism | Self::ArtisticInterests | Self::Introspection => Trait::OpennessToExperience,
            Self::Manipulativeness | Self::Honesty | Self::Fairness => Trait::Integrity,
            Self::ExcitementSeeking | Self::Cautiousness | Self::Immoderation | Self::Recklessness => Trait::Impulsivity,
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
        let cleaned = s.trim().to_lowercase();
        if cleaned.contains("neuroticism") {
            Some(Self::Neuroticism)
        } else if cleaned.contains("sociability") {
            Some(Self::Sociability)
        } else if cleaned.contains("conscientiousness") {
            Some(Self::Conscientiousness)
        } else if cleaned.contains("integrity") {
            Some(Self::Integrity)
        } else if cleaned.contains("openness") {
            Some(Self::OpennessToExperience)
        } else if cleaned.contains("impulsivity") {
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
}

impl MetaTrait {
    pub const ALL: [MetaTrait; 3] = [
        MetaTrait::Stability,
        MetaTrait::Plasticity,
        MetaTrait::Disinhibition,
    ];

    pub fn parse(s: &str) -> Option<Self> {
        let cleaned = s.trim().to_lowercase();
        if cleaned.contains("stability") {
            Some(Self::Stability)
        } else if cleaned.contains("plasticity") {
            Some(Self::Plasticity)
        } else if cleaned.contains("disinhibition") {
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

    /// Standard Error (SE) = \frac{\sqrt{\sum w_i^2}}{\sum |w_i|}
    pub fn standard_error(&self) -> Option<f32> {
        if self.total_abs_weight == 0.0 {
            None
        } else {
            Some(self.total_sq_weight.sqrt() / self.total_abs_weight)
        }
    }

    pub fn tier(&self) -> Option<ScoreTier> {
        self.normalized_score().map(ScoreTier::from_score)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        };
        state.rebuild_cache();
        state
    }

    /// Fully rebuilds accumulators from question list.
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

        // 3. Rebuild pending queue from unanswered questions
        let unanswered: Vec<usize> = self
            .questions
            .iter()
            .enumerate()
            .filter(|(_, q)| q.response.is_none())
            .map(|(i, _)| i)
            .collect();

        // Keep current queue order where possible, retaining only unanswered
        let mut new_queue = VecDeque::new();
        for &idx in &self.pending_queue {
            if idx < self.questions.len() && self.questions[idx].response.is_none() && !new_queue.contains(&idx) {
                new_queue.push_back(idx);
            }
        }
        for idx in unanswered {
            if !new_queue.contains(&idx) {
                new_queue.push_back(idx);
            }
        }
        self.pending_queue = new_queue;

        if let Some(&first_pending) = self.pending_queue.front()
            && (self.questions.get(self.current_focus_idx).is_none() || self.questions[self.current_focus_idx].response.is_some())
        {
            self.current_focus_idx = first_pending;
        }
    }

    /// Sets response for a given question index and updates score accumulators.
    pub fn answer_question(&mut self, question_idx: usize, response: Response) -> bool {
        if question_idx >= self.questions.len() {
            return false;
        }

        let q = &mut self.questions[question_idx];
        let old_response = q.response;
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

        true
    }

    /// Clears response for a question.
    pub fn clear_response(&mut self, question_idx: usize) -> bool {
        if question_idx >= self.questions.len() {
            return false;
        }

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
            true
        } else {
            false
        }
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
        if let Some(pos) = self.pending_queue.iter().position(|&idx| idx == self.current_focus_idx) {
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

    /// Number of questions answered.
    pub fn answered_count(&self) -> usize {
        self.questions.iter().filter(|q| q.response.is_some()).count()
    }

    /// Total questions count.
    pub fn total_questions(&self) -> usize {
        self.questions.len()
    }

    /// Completion rate in [0.0, 1.0].
    pub fn completion_rate(&self) -> f32 {
        if self.questions.is_empty() {
            0.0
        } else {
            self.answered_count() as f32 / self.questions.len() as f32
        }
    }

    /// Checks if all questions have been answered.
    pub fn is_completed(&self) -> bool {
        !self.questions.is_empty() && self.answered_count() == self.questions.len()
    }

    /// Resets all answers and restores initial state.
    pub fn reset(&mut self) {
        *self = Self::from_embedded_data();
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

/// Parses the embedded CSV files and returns the questions ordered according to `Optimized_Keys.csv`.
pub fn load_optimized_questions() -> Vec<Question> {
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
    let opt_lines: Vec<&str> = OPTIMIZED_KEYS_CSV.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
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

pub fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in line.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }
    fields.push(current.trim().to_string());
    fields
}
