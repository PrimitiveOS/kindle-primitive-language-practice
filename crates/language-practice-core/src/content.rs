pub struct Lesson {
    pub title: &'static str,
    pub phrase: &'static str,
    pub meaning: &'static str,
    pub exercises: [Exercise; 3],
}

pub struct Exercise {
    pub instruction: &'static str,
    pub prompt: &'static str,
    pub activity: Activity,
    pub explanation: &'static str,
}

pub enum Activity {
    Choose {
        options: [&'static str; 3],
        correct: usize,
    },
    Order {
        words: &'static [&'static str],
    },
}

macro_rules! choose {
    ($instruction:expr, $prompt:expr, [$a:expr,$b:expr,$c:expr], $correct:expr, $explanation:expr) => {
        Exercise {
            instruction: $instruction,
            prompt: $prompt,
            activity: Activity::Choose {
                options: [$a, $b, $c],
                correct: $correct,
            },
            explanation: $explanation,
        }
    };
}

macro_rules! cloze {
    ($prompt:expr, $choices:tt, $correct:expr, $explanation:expr) => {
        choose!(
            "Chọn từ điền vào chỗ trống.",
            $prompt,
            $choices,
            $correct,
            $explanation
        )
    };
}

macro_rules! order {
    ($prompt:expr, $words:expr, $explanation:expr) => {
        Exercise {
            instruction: "Chạm vào các từ theo đúng thứ tự.",
            prompt: $prompt,
            activity: Activity::Order { words: &$words },
            explanation: $explanation,
        }
    };
}

pub const LESSONS: [Lesson; 10] = [
    Lesson {
        title: "Chào hỏi",
        phrase: "Good morning.",
        meaning: "Chào buổi sáng.",
        exercises: [
            choose!(
                "Chọn nghĩa tiếng Việt.",
                "Good morning.",
                ["Chào buổi tối.", "Chào buổi sáng.", "Tạm biệt."],
                1,
                "Good morning là lời chào vào buổi sáng."
            ),
            cloze!(
                "I ___ Lan.",
                ["am", "is", "are"],
                0,
                "Dùng am với chủ ngữ I. I am Lan = Tôi là Lan."
            ),
            order!(
                "Rất vui được gặp bạn.",
                ["Nice", "to", "meet", "you"],
                "Nice to meet you là lời chào khi mới gặp."
            ),
        ],
    },
    Lesson {
        title: "Giới thiệu",
        phrase: "My name is Minh.",
        meaning: "Tên tôi là Minh.",
        exercises: [
            choose!(
                "Chọn câu tiếng Anh có nghĩa là «Bạn tên gì?»",
                "Bạn tên gì?",
                ["Where are you from?", "How are you?", "What is your name?"],
                2,
                "What is your name? là cách hỏi tên."
            ),
            cloze!(
                "My name ___ Minh.",
                ["are", "is", "am"],
                1,
                "Name là danh từ số ít. Dùng is trong câu này."
            ),
            order!(
                "Tôi đến từ Việt Nam.",
                ["I", "am", "from", "Vietnam"],
                "Dùng I am from + tên địa điểm để nói mình đến từ đâu."
            ),
        ],
    },
    Lesson {
        title: "Trong lớp học",
        phrase: "Please open your book.",
        meaning: "Hãy mở sách của bạn.",
        exercises: [
            choose!(
                "Chọn nghĩa tiếng Việt.",
                "Please open your book.",
                ["Hãy mở sách của bạn.", "Hãy đóng cửa.", "Hãy viết tên bạn."],
                0,
                "Open là mở, và your book là sách của bạn."
            ),
            cloze!(
                "This is ___ pencil.",
                ["an", "are", "a"],
                2,
                "Pencil bắt đầu bằng âm phụ âm, nên dùng a pencil."
            ),
            order!(
                "Hãy lắng nghe tôi.",
                ["Please", "listen", "to", "me"],
                "Dùng listen to khi nói lắng nghe ai hoặc điều gì."
            ),
        ],
    },
    Lesson {
        title: "Gia đình",
        phrase: "She is my sister.",
        meaning: "Cô ấy là chị hoặc em gái tôi.",
        exercises: [
            choose!(
                "Chọn nghĩa tiếng Việt.",
                "She is my sister.",
                [
                    "Đó là bố tôi.",
                    "Cô ấy là chị hoặc em gái tôi.",
                    "Cậu ấy là bạn tôi."
                ],
                1,
                "Sister chỉ chị gái hoặc em gái."
            ),
            cloze!(
                "He ___ my brother.",
                ["am", "are", "is"],
                2,
                "Dùng is với he. Brother chỉ anh trai hoặc em trai."
            ),
            order!(
                "Đây là mẹ tôi.",
                ["This", "is", "my", "mother"],
                "This is dùng để giới thiệu một người hoặc một vật."
            ),
        ],
    },
    Lesson {
        title: "Đồ ăn và thức uống",
        phrase: "I would like some water.",
        meaning: "Tôi muốn một ít nước.",
        exercises: [
            choose!(
                "Chọn nghĩa tiếng Việt.",
                "I would like some water.",
                [
                    "Tôi muốn một ít nước.",
                    "Tôi có một quả táo.",
                    "Tôi muốn một ít cơm."
                ],
                0,
                "I would like là cách lịch sự để nói mình muốn gì."
            ),
            cloze!(
                "I have ___ apple.",
                ["a", "an", "am"],
                1,
                "Apple bắt đầu bằng âm nguyên âm, nên dùng an apple."
            ),
            order!(
                "Tôi thích cơm.",
                ["I", "like", "rice"],
                "Đặt món ăn sau like để nói bạn thích món đó."
            ),
        ],
    },
    Lesson {
        title: "Giờ giấc",
        phrase: "I study at seven.",
        meaning: "Tôi học lúc bảy giờ.",
        exercises: [
            choose!(
                "Chọn nghĩa tiếng Việt.",
                "It is seven fifteen.",
                [
                    "Bây giờ là năm giờ.",
                    "Bây giờ là bảy giờ.",
                    "Bây giờ là bảy giờ mười lăm."
                ],
                2,
                "Seven fifteen là 7 giờ 15 phút."
            ),
            cloze!(
                "I study ___ seven.",
                ["at", "in", "on"],
                0,
                "Dùng at trước một giờ cụ thể."
            ),
            order!(
                "Bây giờ là sáu giờ.",
                ["It", "is", "six", "o'clock"],
                "It is + giờ dùng để nói thời gian hiện tại."
            ),
        ],
    },
    Lesson {
        title: "Chỉ đường",
        phrase: "Turn left here.",
        meaning: "Rẽ trái ở đây.",
        exercises: [
            choose!(
                "Chọn nghĩa tiếng Việt.",
                "Turn left here.",
                ["Đi thẳng ở đây.", "Rẽ trái ở đây.", "Rẽ phải ở đây."],
                1,
                "Left là bên trái. Turn left nghĩa là rẽ trái."
            ),
            cloze!(
                "The bank is ___ the school. (Ngân hàng ở đối diện trường học.)",
                ["behind", "inside", "across from"],
                2,
                "Across from nghĩa là ở đối diện."
            ),
            order!(
                "Trạm xe buýt ở đây.",
                ["The", "bus", "stop", "is", "here"],
                "Bus stop là trạm xe buýt, và here nghĩa là ở đây."
            ),
        ],
    },
    Lesson {
        title: "Thói quen hằng ngày",
        phrase: "I read every evening.",
        meaning: "Tôi đọc sách mỗi tối.",
        exercises: [
            choose!(
                "Chọn nghĩa tiếng Việt.",
                "I read every evening.",
                [
                    "Tôi đọc sách mỗi tối.",
                    "Tôi đi bộ mỗi sáng.",
                    "Tôi viết mỗi chiều."
                ],
                0,
                "Every evening nghĩa là mỗi tối."
            ),
            cloze!(
                "I wake ___ at six. (Tôi thức dậy lúc sáu giờ.)",
                ["to", "up", "of"],
                1,
                "Wake up là cụm từ chỉ việc thức dậy."
            ),
            order!(
                "Tôi đi học.",
                ["I", "go", "to", "school"],
                "Go to school nghĩa là đi học."
            ),
        ],
    },
    Lesson {
        title: "Mua sắm",
        phrase: "How much is this?",
        meaning: "Cái này giá bao nhiêu?",
        exercises: [
            choose!(
                "Chọn nghĩa tiếng Việt.",
                "How much is this?",
                [
                    "Cái này ở đâu?",
                    "Cái này có màu gì?",
                    "Cái này giá bao nhiêu?"
                ],
                2,
                "Dùng How much để hỏi giá."
            ),
            cloze!(
                "Can I pay ___ card? (Tôi có thể trả bằng thẻ không?)",
                ["by", "on", "in"],
                0,
                "Pay by card nghĩa là trả bằng thẻ."
            ),
            order!(
                "Tôi cần một cái túi.",
                ["I", "need", "a", "bag"],
                "Dùng I need khi bạn cần một thứ gì đó."
            ),
        ],
    },
    Lesson {
        title: "Thời tiết",
        phrase: "It is raining.",
        meaning: "Trời đang mưa.",
        exercises: [
            choose!(
                "Chọn nghĩa tiếng Việt.",
                "It is raining.",
                ["Trời đang nắng.", "Trời đang mưa.", "Trời đang lạnh."],
                1,
                "Raining diễn tả mưa đang diễn ra."
            ),
            cloze!(
                "It ___ cold today.",
                ["am", "are", "is"],
                2,
                "Dùng It is để mô tả thời tiết, như It is cold."
            ),
            order!(
                "Hôm nay trời lạnh.",
                ["It", "is", "cold", "today"],
                "Cold là lạnh và today nghĩa là hôm nay."
            ),
        ],
    },
];
