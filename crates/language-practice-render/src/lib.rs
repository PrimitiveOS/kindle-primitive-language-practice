use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use language_practice_core::{word_bank, Activity, Study, LESSONS, PAGE_SIZE};
use primitive_app::{App, Change, WIDTH};
use primitive_ui::{
    back_button, header, screen_rect, Canvas, Input, Rect, TextStyle, BODY, CAPTION, PAPER,
};

#[derive(Clone, Copy)]
enum Screen {
    Lessons,
    Review,
    Intro,
    Practice,
}

pub struct View {
    pub study: Study,
    screen: Screen,
    page: usize,
    path: Option<PathBuf>,
    pending_save: bool,
}

impl View {
    pub fn open(path: PathBuf) -> Self {
        let mut view = Self::preview();
        view.study = load(&path).unwrap_or_default();
        view.page = view
            .study
            .visit()
            .map_or(0, |active| active.lesson / PAGE_SIZE);
        view.path = Some(path);
        view
    }

    pub fn preview() -> Self {
        Self {
            study: Study::default(),
            screen: Screen::Lessons,
            page: 0,
            path: None,
            pending_save: false,
        }
    }

    fn persist(&mut self) {
        self.pending_save = true;
        let _ = self.save();
    }

    fn resumable(&self) -> bool {
        self.study
            .current()
            .is_some_and(|session| !session.finished())
            || self.study.next_lesson().is_some()
    }

    fn list(&self, review: bool) -> Vec<usize> {
        (self.page * PAGE_SIZE..(self.page + 1) * PAGE_SIZE)
            .filter(|&index| {
                !review
                    || self.study.mistakes(index) > 0
                    || self.study.at(index, true).is_some_and(|s| !s.finished())
            })
            .collect()
    }

    fn open_lesson(&mut self, index: usize, review: bool) -> bool {
        let new = self.study.at(index, review).is_none();
        if !self.study.open(index, review) {
            return false;
        }
        self.page = index / PAGE_SIZE;
        self.screen = if new && !review {
            Screen::Intro
        } else {
            Screen::Practice
        };
        self.persist();
        true
    }

    fn draw_list(&self, canvas: &mut Canvas, w: i32, review: bool) {
        header(
            canvas,
            if review {
                "Ôn lỗi"
            } else {
                "Language Practice"
            },
            if review { "Quay lại" } else { "Xong" },
        );
        if review {
            canvas.text(
                screen_rect(w, 40, 224, 1184, 70),
                "Chọn bài để ôn lại các câu sai.",
                BODY,
            );
        } else {
            if self.resumable() {
                canvas.button(left(w, 205), "Tiếp tục", true, false);
            }
            canvas.button(
                right(w, 205),
                &format!("Ôn lỗi ({})", self.study.total_mistakes()),
                false,
                false,
            );
        }

        for (page, label) in ["Bài 1–5", "Bài 6–10"].into_iter().enumerate() {
            canvas.button(tab(w, page), label, page == self.page, false);
        }
        let visible = self.list(review);
        for (row, index) in visible.iter().copied().enumerate() {
            canvas.button(lesson_button(w, row), LESSONS[index].title, false, false);
            let status = if review {
                format!("{} câu sai", self.study.mistakes(index))
            } else {
                match self.study.at(index, false) {
                    Some(session) if session.finished() => "Đã xong".into(),
                    Some(_) => "Đang học".into(),
                    None => format!("Bài {}", index + 1),
                }
            };
            canvas.text(
                screen_rect(w, 1010, 544 + row as i32 * 154, 175, 42),
                &status,
                CAPTION.right(),
            );
        }
        if visible.is_empty() {
            canvas.text(
                screen_rect(w, 40, 617, 1184, 140),
                "Chưa có câu sai trong nhóm bài này.",
                BODY.center().lines(3),
            );
        }
    }

    fn draw_intro(&self, canvas: &mut Canvas, w: i32) {
        let Some(visit) = self.study.visit() else {
            return;
        };
        let lesson = &LESSONS[visit.lesson];
        header(canvas, "Language Practice", "Quay lại");
        canvas.text(
            screen_rect(w, 40, 212, 1184, 88),
            lesson.title,
            TextStyle::new(50).bold(),
        );
        canvas.panel(screen_rect(w, 40, 391, 1184, 600), 22, 2, PAPER);
        canvas.text(screen_rect(w, 88, 466, 1088, 64), "Câu mẫu", CAPTION);
        canvas.text(
            screen_rect(w, 88, 571, 1088, 154),
            lesson.phrase,
            TextStyle::new(54).bold().lines(2),
        );
        canvas.text(
            screen_rect(w, 88, 752, 1088, 141),
            lesson.meaning,
            BODY.lines(3),
        );
        canvas.button(left(w, 1450), "Bắt đầu", true, false);
        canvas.text(
            screen_rect(w, 660, 1484, 564, 60),
            "Ba câu luyện tập",
            CAPTION,
        );
    }

    fn draw_practice(&self, canvas: &mut Canvas, w: i32) {
        let (Some(visit), Some(session)) = (self.study.visit(), self.study.current()) else {
            return;
        };
        header(canvas, "Language Practice", "Quay lại");
        canvas.text(
            screen_rect(w, 40, 200, 1184, 65),
            LESSONS[visit.lesson].title,
            BODY.bold(),
        );
        let Some(exercise) = self.study.exercise() else {
            let (score, total) = session.score();
            let title = if visit.review {
                "Ôn tập xong"
            } else {
                "Hoàn thành bài"
            };
            canvas.text(
                screen_rect(w, 40, 411, 1184, 93),
                title,
                TextStyle::new(64).bold().center(),
            );
            canvas.text(
                screen_rect(w, 40, 542, 1184, 74),
                &format!("Đúng {score}/{total} câu"),
                BODY.center(),
            );
            if !visit.review {
                canvas.button(left(w, 940), "Học lại", false, false);
                if visit.lesson + 1 < LESSONS.len() {
                    canvas.button(right(w, 940), "Bài tiếp theo", true, false);
                }
            }
            canvas.button(
                left(w, 1450),
                if visit.review {
                    "Ôn lỗi"
                } else {
                    "Danh sách bài"
                },
                true,
                false,
            );
            return;
        };

        let (number, total) = session.position();
        canvas.text(
            screen_rect(w, 40, 267, 1184, 47),
            &format!("Câu {number}/{total}"),
            CAPTION,
        );
        canvas.text(
            screen_rect(w, 40, 338, 1184, 87),
            exercise.instruction,
            BODY.lines(2),
        );
        canvas.text(
            screen_rect(w, 40, 440, 1184, 142),
            exercise.prompt,
            TextStyle::new(45).bold().lines(3),
        );
        match &exercise.activity {
            Activity::Choose { options, .. } => {
                for (i, option) in options.iter().enumerate() {
                    canvas.button(
                        choice_button(w, i),
                        option,
                        session.choice == Some(i),
                        false,
                    );
                }
            }
            Activity::Order { words } => {
                let selected = session
                    .draft
                    .iter()
                    .map(|&index| words[index])
                    .collect::<Vec<_>>()
                    .join(" ");
                canvas.panel(screen_rect(w, 40, 601, 1184, 116), 16, 2, PAPER);
                if selected.is_empty() {
                    canvas.text(screen_rect(w, 68, 638, 1128, 50), "Câu của bạn", CAPTION);
                } else {
                    canvas.text(screen_rect(w, 68, 630, 1128, 74), &selected, BODY);
                }
                for (i, slot) in word_bank(visit.lesson, words.len()).into_iter().enumerate() {
                    canvas.button(
                        word_button(w, i),
                        words[slot],
                        session.draft.contains(&slot),
                        false,
                    );
                }
                if session.result().is_none() {
                    if !session.draft.is_empty() {
                        canvas.button(right(w, 1450), "Bỏ từ cuối", false, false);
                    }
                    if session.draft.len() == words.len() {
                        canvas.button(left(w, 1450), "Kiểm tra", true, false);
                    }
                }
            }
        }
        if let Some(correct) = session.result() {
            let answer = match &exercise.activity {
                Activity::Choose { options, correct } => options[*correct].to_string(),
                Activity::Order { words } => words.join(" "),
            };
            canvas.panel(screen_rect(w, 40, 1066, 1184, 310), 22, 2, PAPER);
            canvas.text(
                screen_rect(w, 68, 1090, 1128, 58),
                if correct { "Đúng" } else { "Chưa đúng" },
                BODY.bold(),
            );
            canvas.text(screen_rect(w, 68, 1158, 1128, 60), &answer, BODY.bold());
            canvas.text(
                screen_rect(w, 68, 1230, 1128, 132),
                exercise.explanation,
                BODY.lines(3),
            );
            canvas.button(left(w, 1450), "Tiếp theo", true, false);
        }
    }

    fn tap(&mut self, x: f32, y: f32) -> Change {
        let w = WIDTH as i32;
        if back_button(w).contains(x, y) {
            if matches!(self.screen, Screen::Lessons) {
                return Change::Exit;
            }
            self.back();
            return Change::Full;
        }
        match self.screen {
            Screen::Lessons | Screen::Review => {
                let review = matches!(self.screen, Screen::Review);
                if !review && right(w, 205).contains(x, y) {
                    self.screen = Screen::Review;
                    self.page = (0..LESSONS.len())
                        .find(|&i| self.study.mistakes(i) > 0)
                        .map_or(0, |i| i / PAGE_SIZE);
                    return Change::Full;
                }
                if !review && left(w, 205).contains(x, y) {
                    if self.study.current().is_some_and(|s| !s.finished()) {
                        self.screen = Screen::Practice;
                        return Change::Full;
                    }
                    if let Some(index) = self.study.next_lesson() {
                        if self.open_lesson(index, false) {
                            return Change::Full;
                        }
                    }
                }
                if let Some(page) = (0..2).find(|&i| tab(w, i).contains(x, y)) {
                    self.page = page;
                    return Change::Partial;
                }
                if let Some((_, index)) = self
                    .list(review)
                    .into_iter()
                    .enumerate()
                    .find(|(i, _)| lesson_button(w, *i).contains(x, y))
                {
                    if self.open_lesson(index, review) {
                        return Change::Full;
                    }
                }
                Change::None
            }
            Screen::Intro => {
                if left(w, 1450).contains(x, y) {
                    self.screen = Screen::Practice;
                    Change::Full
                } else {
                    Change::None
                }
            }
            Screen::Practice => self.practice_tap(w, x, y),
        }
    }

    fn practice_tap(&mut self, w: i32, x: f32, y: f32) -> Change {
        let (Some(visit), Some(session)) = (self.study.visit(), self.study.current()) else {
            return Change::None;
        };
        if session.finished() {
            if left(w, 1450).contains(x, y) {
                self.back();
                return Change::Full;
            }
            if !visit.review && left(w, 940).contains(x, y) && self.study.restart() {
                self.screen = Screen::Intro;
                self.persist();
                return Change::Full;
            }
            if !visit.review
                && right(w, 940).contains(x, y)
                && self.open_lesson(visit.lesson + 1, false)
            {
                return Change::Full;
            }
            return Change::None;
        }
        if session.result().is_some() {
            if left(w, 1450).contains(x, y) && self.study.advance() {
                self.persist();
                return Change::Full;
            }
            return Change::None;
        }
        let Some(exercise) = self.study.exercise() else {
            return Change::None;
        };
        let changed = match &exercise.activity {
            Activity::Choose { .. } => (0..3)
                .find(|&i| choice_button(w, i).contains(x, y))
                .is_some_and(|i| self.study.choose(i)),
            Activity::Order { words } => {
                if left(w, 1450).contains(x, y) {
                    self.study.check_words()
                } else if right(w, 1450).contains(x, y) {
                    self.study.undo_word()
                } else {
                    word_bank(visit.lesson, words.len())
                        .into_iter()
                        .enumerate()
                        .find(|(i, _)| word_button(w, *i).contains(x, y))
                        .is_some_and(|(_, slot)| self.study.add_word(slot))
                }
            }
        };
        if changed {
            self.persist();
            Change::Partial
        } else {
            Change::None
        }
    }

    fn back(&mut self) {
        self.screen = if matches!(self.screen, Screen::Practice)
            && self.study.visit().is_some_and(|v| v.review)
        {
            Screen::Review
        } else {
            Screen::Lessons
        };
    }
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        let w = canvas.bounds().w;
        match self.screen {
            Screen::Lessons => self.draw_list(canvas, w, false),
            Screen::Review => self.draw_list(canvas, w, true),
            Screen::Intro => self.draw_intro(canvas, w),
            Screen::Practice => self.draw_practice(canvas, w),
        }
        if self.pending_save {
            canvas.text(
                screen_rect(w, 40, 1588, 1184, 60),
                "Chưa lưu được tiến độ.",
                CAPTION,
            );
        }
    }

    fn input(&mut self, input: Input) -> Change {
        match input {
            Input::Tap(x, y) => self.tap(x, y),
            Input::PageBack if matches!(self.screen, Screen::Practice | Screen::Intro) => {
                self.back();
                Change::Full
            }
            Input::PageForward
                if matches!(self.screen, Screen::Lessons | Screen::Review) && self.page == 0 =>
            {
                self.page = 1;
                Change::Partial
            }
            Input::PageBack if self.page == 1 => {
                self.page = 0;
                Change::Partial
            }
            _ => Change::None,
        }
    }

    fn save(&mut self) -> io::Result<()> {
        if self.pending_save {
            if let Some(path) = &self.path {
                primitive_app::save(path, &self.study.pack())?;
            }
            self.pending_save = false;
        }
        Ok(())
    }

    fn storage_returned(&mut self) -> io::Result<()> {
        self.save()
    }
}

fn load(path: &Path) -> Option<Study> {
    primitive_app::with_storage(|| {
        let mut bytes = Vec::new();
        File::open(path)?.take(32_769).read_to_end(&mut bytes)?;
        Ok(Study::unpack(&bytes).filter(|_| bytes.len() <= 32_768))
    })
    .ok()
    .flatten()
}

fn left(w: i32, y: i32) -> Rect {
    screen_rect(w, 40, y, 564, 110)
}

fn right(w: i32, y: i32) -> Rect {
    screen_rect(w, 660, y, 564, 110)
}

fn tab(w: i32, page: usize) -> Rect {
    if page == 0 {
        left(w, 354)
    } else {
        right(w, 354)
    }
}

fn lesson_button(w: i32, index: usize) -> Rect {
    screen_rect(w, 40, 505 + index as i32 * 154, 1184, 120)
}

fn choice_button(w: i32, index: usize) -> Rect {
    screen_rect(w, 40, 614 + index as i32 * 144, 1184, 116)
}

fn word_button(w: i32, index: usize) -> Rect {
    screen_rect(
        w,
        40 + (index % 3) as i32 * 404,
        746 + (index / 3) as i32 * 135,
        376,
        114,
    )
}
