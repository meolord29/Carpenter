//! Course models: the authored spec + serialized row/list/counts shapes.

use serde::{Deserialize, Serialize};

/// Authored course definition (`docs/specs/02-course-spec.md`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CourseSpec {
    /// Course title (required, non-empty).
    pub title: String,
    /// Slug; derived from `title` if absent.
    pub slug: Option<String>,
    /// Course mission goal (required, non-empty).
    pub goal: String,
    /// Longer description (optional, default empty).
    #[serde(default)]
    pub description: String,
}

/// A full course_meta row (mirrors `course.json`); echoed as `updated:` on update.
#[derive(Debug, Clone, Serialize)]
pub struct CourseRow {
    /// slug (PK).
    pub slug: String,
    /// title.
    pub title: String,
    /// goal.
    pub goal: String,
    /// description.
    pub description: String,
    /// created_at (ISO-8601 UTC).
    pub created_at: String,
}

/// One element of `course list`.
#[derive(Debug, Clone, Serialize)]
pub struct CourseListItem {
    /// slug.
    pub slug: String,
    /// title.
    pub title: String,
    /// goal.
    pub goal: String,
    /// number of lessons.
    pub lessons_count: i64,
}

/// Per-table counts for `course show`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CourseCounts {
    /// lessons.
    pub lessons: i64,
    /// sections.
    pub sections: i64,
    /// practice.
    pub practice: i64,
    /// quizzes.
    pub quizzes: i64,
}

/// Representative examples for spec generation (adr/008).
pub mod examples {
    use super::CourseCounts;
    use super::CourseListItem;
    use super::CourseRow;
    use super::CourseSpec;
    use crate::models::Data;

    /// A representative `CourseSpec`.
    pub fn spec() -> CourseSpec {
        CourseSpec {
            title: String::from("Data Structures"),
            slug: Some(String::from("data-structures")),
            goal: String::from("Understand core data structures from the ground up"),
            description: String::from("…"),
        }
    }

    /// `(cmd, input, note, data)` rows for the `course` output contract.
    pub fn rows() -> Vec<(&'static str, &'static str, &'static str, Data)> {
        let d = String::from("…");
        vec![
            (
                "create --spec -",
                "CourseSpec",
                "`AlreadyExists` on duplicate slug",
                Data::CourseCreate {
                    slug: String::from("data-structures"),
                    title: String::from("Data Structures"),
                    path: String::from("<root>/courses/data-structures"),
                },
            ),
            (
                "list",
                "—",
                "",
                Data::CourseList {
                    courses: vec![CourseListItem {
                        slug: d.clone(),
                        title: d.clone(),
                        goal: d.clone(),
                        lessons_count: 0,
                    }],
                    errors: vec![],
                },
            ),
            (
                "show <slug>",
                "—",
                "`NotFound` if absent",
                Data::CourseShow {
                    slug: d.clone(),
                    title: d.clone(),
                    goal: d.clone(),
                    description: d.clone(),
                    counts: CourseCounts::default(),
                },
            ),
            (
                "update <slug> --spec - --force",
                "CourseSpec",
                "`Conflict` without `--force`",
                Data::CourseUpdate {
                    slug: d.clone(),
                    updated: CourseRow {
                        slug: d.clone(),
                        title: d.clone(),
                        goal: d.clone(),
                        description: d.clone(),
                        created_at: String::from("2026-08-09T12:00:00Z"),
                    },
                },
            ),
            (
                "delete <slug> --force",
                "—",
                "`Conflict` without `--force`",
                Data::CourseDelete {
                    slug: d.clone(),
                    deleted: true,
                },
            ),
            (
                "switch <slug>",
                "—",
                "writes config",
                Data::CourseSwitch {
                    active_course: d.clone(),
                },
            ),
        ]
    }
}
