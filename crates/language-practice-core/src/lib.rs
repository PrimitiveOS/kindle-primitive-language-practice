use serde::{Deserialize, Serialize};

mod content;
pub use content::{Activity, Exercise, Lesson, LESSONS};

pub const PAGE_SIZE: usize = 5;
const EXERCISE_COUNT: usize = 3;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Visit {
    pub lesson: usize,
    pub review: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    queue: Vec<usize>,
    step: usize,
    answers: [Option<bool>; EXERCISE_COUNT],
    pub choice: Option<usize>,
    pub draft: Vec<usize>,
}

impl Session {
    fn new(queue: Vec<usize>) -> Self {
        Self {
            queue,
            step: 0,
            answers: [None; EXERCISE_COUNT],
            choice: None,
            draft: Vec::new(),
        }
    }

    pub fn exercise(&self) -> Option<usize> {
        self.queue.get(self.step).copied()
    }

    pub fn position(&self) -> (usize, usize) {
        (self.step + 1, self.queue.len())
    }

    pub fn result(&self) -> Option<bool> {
        self.answers[self.exercise()?]
    }

    pub fn finished(&self) -> bool {
        self.step >= self.queue.len()
    }

    pub fn score(&self) -> (usize, usize) {
        (
            self.queue
                .iter()
                .filter(|&&i| self.answers[i] == Some(true))
                .count(),
            self.queue.len(),
        )
    }

    fn valid(&self, lesson: usize) -> bool {
        if self.queue.is_empty()
            || self.queue.len() > EXERCISE_COUNT
            || self.step > self.queue.len()
            || !self.queue.iter().enumerate().all(|(i, &slot)| {
                slot < EXERCISE_COUNT
                    && !self.queue[..i].contains(&slot)
                    && (i >= self.step || self.answers[slot].is_some())
            })
        {
            return false;
        }
        let Some(index) = self.exercise() else {
            return self.choice.is_none() && self.draft.is_empty();
        };
        match LESSONS[lesson].exercises[index].activity {
            Activity::Choose { options, .. } => {
                self.draft.is_empty() && self.choice.is_none_or(|c| c < options.len())
            }
            Activity::Order { words } => {
                self.choice.is_none()
                    && self.draft.len() <= words.len()
                    && self
                        .draft
                        .iter()
                        .enumerate()
                        .all(|(i, &slot)| slot < words.len() && !self.draft[..i].contains(&slot))
            }
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Study {
    active: Option<Visit>,
    lessons: [Option<Session>; LESSONS.len()],
    reviews: [Option<Session>; LESSONS.len()],
    mistakes: [[bool; EXERCISE_COUNT]; LESSONS.len()],
}

impl Study {
    pub fn pack(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("language progress is serializable")
    }

    pub fn unpack(bytes: &[u8]) -> Option<Self> {
        let state: Self = serde_json::from_slice(bytes).ok()?;
        for sessions in [&state.lessons, &state.reviews] {
            if sessions
                .iter()
                .enumerate()
                .any(|(i, s)| s.as_ref().is_some_and(|s| !s.valid(i)))
            {
                return None;
            }
        }
        if state.active.is_some() && state.current().is_none() {
            return None;
        }
        Some(state)
    }

    pub fn visit(&self) -> Option<Visit> {
        self.active
    }

    pub fn at(&self, lesson: usize, review: bool) -> Option<&Session> {
        let sessions = if review { &self.reviews } else { &self.lessons };
        sessions.get(lesson)?.as_ref()
    }

    pub fn current(&self) -> Option<&Session> {
        let visit = self.active?;
        self.at(visit.lesson, visit.review)
    }

    fn current_mut(&mut self) -> Option<&mut Session> {
        let visit = self.active?;
        let sessions = if visit.review {
            &mut self.reviews
        } else {
            &mut self.lessons
        };
        sessions.get_mut(visit.lesson)?.as_mut()
    }

    pub fn next_lesson(&self) -> Option<usize> {
        (0..LESSONS.len()).find(|&i| self.at(i, false).is_none_or(|s| !s.finished()))
    }

    pub fn mistakes(&self, lesson: usize) -> usize {
        self.mistakes
            .get(lesson)
            .map_or(0, |items| items.iter().filter(|&&x| x).count())
    }

    pub fn total_mistakes(&self) -> usize {
        (0..LESSONS.len()).map(|i| self.mistakes(i)).sum()
    }

    pub fn open(&mut self, lesson: usize, review: bool) -> bool {
        if lesson >= LESSONS.len() {
            return false;
        }
        let sessions = if review {
            &mut self.reviews
        } else {
            &mut self.lessons
        };
        let slot = &mut sessions[lesson];
        let start = slot.is_none() || (review && slot.as_ref().is_some_and(Session::finished));
        if start {
            let queue = if review {
                (0..EXERCISE_COUNT)
                    .filter(|&i| self.mistakes[lesson][i])
                    .collect::<Vec<_>>()
            } else {
                (0..EXERCISE_COUNT).collect()
            };
            if queue.is_empty() {
                return false;
            }
            *slot = Some(Session::new(queue));
        }
        self.active = Some(Visit { lesson, review });
        true
    }

    pub fn restart(&mut self) -> bool {
        let Some(visit) = self.active else {
            return false;
        };
        if visit.review || !self.current().is_some_and(Session::finished) {
            return false;
        }
        self.lessons[visit.lesson] = Some(Session::new((0..EXERCISE_COUNT).collect()));
        true
    }

    pub fn exercise(&self) -> Option<&'static Exercise> {
        let lesson = self.active?.lesson;
        Some(&LESSONS[lesson].exercises[self.current()?.exercise()?])
    }

    pub fn choose(&mut self, index: usize) -> bool {
        let Some(Exercise {
            activity: Activity::Choose { options, correct },
            ..
        }) = self.exercise()
        else {
            return false;
        };
        if index >= options.len() || self.current().is_none_or(|s| s.result().is_some()) {
            return false;
        }
        self.current_mut().unwrap().choice = Some(index);
        self.mark(index == *correct)
    }

    pub fn add_word(&mut self, index: usize) -> bool {
        let Some(Exercise {
            activity: Activity::Order { words },
            ..
        }) = self.exercise()
        else {
            return false;
        };
        let Some(session) = self.current_mut() else {
            return false;
        };
        if session.result().is_some() || index >= words.len() || session.draft.contains(&index) {
            return false;
        }
        session.draft.push(index);
        true
    }

    pub fn undo_word(&mut self) -> bool {
        self.current_mut()
            .is_some_and(|s| s.result().is_none() && s.draft.pop().is_some())
    }

    pub fn check_words(&mut self) -> bool {
        let Some(Exercise {
            activity: Activity::Order { words },
            ..
        }) = self.exercise()
        else {
            return false;
        };
        let session = self.current().unwrap();
        if session.draft.len() != words.len() || session.result().is_some() {
            return false;
        }
        self.mark(session.draft.iter().copied().eq(0..words.len()))
    }

    fn mark(&mut self, correct: bool) -> bool {
        let Some((visit, index)) = self.active.zip(self.current().and_then(Session::exercise))
        else {
            return false;
        };
        self.current_mut().unwrap().answers[index] = Some(correct);
        self.mistakes[visit.lesson][index] = !correct;
        true
    }

    pub fn advance(&mut self) -> bool {
        let Some(session) = self.current_mut().filter(|s| s.result().is_some()) else {
            return false;
        };
        session.step += 1;
        session.choice = None;
        session.draft.clear();
        true
    }
}

/// A stable permutation keeps the same word bank after waking or reopening the app.
pub fn word_bank(lesson: usize, word_count: usize) -> Vec<usize> {
    let mut order = (0..word_count).collect::<Vec<_>>();
    let mut seed = (lesson as u64).wrapping_add(1);
    for i in (1..word_count).rev() {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        order.swap(i, ((seed >> 32) as usize) % (i + 1));
    }
    if word_count > 1 && order.iter().copied().eq(0..word_count) {
        order.rotate_left(1);
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_lesson_has_two_selection_questions_and_an_ordering_exercise() {
        assert_eq!(LESSONS.len(), 2 * PAGE_SIZE);
        for (i, lesson) in LESSONS.iter().enumerate() {
            for exercise in &lesson.exercises[..2] {
                let Activity::Choose { options, correct } = exercise.activity else {
                    panic!("selection expected");
                };
                assert!(correct < options.len());
                assert_ne!(options[0], options[1]);
                assert_ne!(options[1], options[2]);
                assert_ne!(options[0], options[2]);
            }
            let Activity::Order { words } = &lesson.exercises[2].activity else {
                panic!("ordering expected");
            };
            let mut bank = word_bank(i, words.len());
            assert!(!bank.iter().copied().eq(0..words.len()));
            bank.sort_unstable();
            assert!(bank.iter().copied().eq(0..words.len()));
        }
    }

    #[test]
    fn review_targets_an_error_without_discarding_a_lesson() {
        let mut study = Study::default();
        assert!(study.open(0, false));
        assert!(study.choose(0));
        assert!(study.advance());
        assert!(study.open(0, true));
        assert_eq!(study.current().unwrap().position(), (1, 1));
        assert!(study.choose(1));
        assert_eq!(study.total_mistakes(), 0);
        assert!(study.advance());
        let mut reopened = Study::unpack(&study.pack()).unwrap();
        assert!(reopened.open(0, false));
        assert_eq!(reopened.current().unwrap().exercise(), Some(1));
    }

    #[test]
    fn sentence_draft_survives_reopening_and_checks_only_when_complete() {
        let mut study = Study::default();
        assert!(study.open(0, false));
        assert!(study.choose(1));
        assert!(study.advance());
        assert!(study.choose(0));
        assert!(study.advance());
        assert!(study.add_word(0));
        assert!(!study.add_word(0));
        assert!(!study.check_words());
        let mut study = Study::unpack(&study.pack()).unwrap();
        assert_eq!(study.current().unwrap().draft, [0]);
        for i in 1..4 {
            assert!(study.add_word(i));
        }
        assert!(study.check_words());
        assert_eq!(study.current().unwrap().result(), Some(true));
        assert!(study.advance());
        assert_eq!(study.next_lesson(), Some(1));
    }
}
