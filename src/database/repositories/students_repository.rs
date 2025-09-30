use crate::models::student::Student;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get a student by email
pub(crate) async fn get_by_email(
    db: &PostgresClient, email: &str,
) -> welds::errors::Result<Option<DbState<Student>>> {
    let mut rows = Student::where_col(|s| s.email.equal(email)).run(db).await?;

    Ok(rows.pop())
}

/// Get a student by student ID
pub(crate) async fn get_by_id(
    db: &PostgresClient, student_id: i32,
) -> welds::errors::Result<Option<DbState<Student>>> {
    let mut rows = Student::where_col(|s| s.student_id.equal(student_id))
        .run(db)
        .await?;

    Ok(rows.pop())
}

/// Get a student by university ID
pub(crate) async fn get_by_university_id(
    db: &PostgresClient, university_id: i32,
) -> welds::errors::Result<Option<DbState<Student>>> {
    let mut rows = Student::where_col(|s| s.university_id.equal(university_id))
        .run(db)
        .await?;

    Ok(rows.pop())
}

/// Get all students
pub(crate) async fn get_all(db: &PostgresClient) -> welds::errors::Result<Vec<DbState<Student>>> {
    Student::all().run(db).await
}

/// Check if an email already exists
pub(crate) async fn email_exists(db: &PostgresClient, email: &str) -> welds::errors::Result<bool> {
    let result = get_by_email(db, email).await?;
    Ok(result.is_some())
}

/// Check if a university ID already exists
pub(crate) async fn university_id_exists(
    db: &PostgresClient, university_id: i32,
) -> welds::errors::Result<bool> {
    let result = get_by_university_id(db, university_id).await?;
    Ok(result.is_some())
}
