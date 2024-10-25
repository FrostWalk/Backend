#[cfg(test)]
mod tests {
    use crate::database::project_options_repository::ProjectOptionsRepository;
    use entity::prelude::ProjectOptions;
    use entity::project_options::ActiveModel as ProjectOptionsActiveModel;
    use sea_orm::ActiveValue::Set;
    use sea_orm::{Database, DatabaseConnection, EntityTrait, NotSet, SqlErr};
    use std::env;

    // Helper function to establish a connection to the test database
    async fn get_test_db_connection() -> DatabaseConnection {
        let db_url = env::var("TEST_DB_URL").expect("TEST_DB_URL must be set in .env");
        Database::connect(&db_url).await.expect("Failed to connect to test database")
    }

    // Helper function to clean up the project_options table after each test
    async fn clean_up_database(db: &DatabaseConnection) {
        ProjectOptions::delete_many().exec(db).await.expect("Failed to clean up database");
        let _ = db.clone().close().await;
    }

    #[tokio::test]
    async fn test_get_all() {
        let db = get_test_db_connection().await;

        // Insert test data
        let repo = ProjectOptionsRepository::new(db.clone());
        repo.create(ProjectOptionsActiveModel {
            id: Set(1),
            name: Set("Option A".to_owned()),
        })
        .await
        .unwrap();

        repo.create(ProjectOptionsActiveModel {
            id: Set(2),
            name: Set("Option B".to_owned()),
        })
        .await
        .unwrap();

        // Execute the test
        let result = repo.get_all().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Option A");
        assert_eq!(result[1].name, "Option B");

        clean_up_database(&db).await;
    }

    #[tokio::test]
    async fn test_get_from_name() {
        let db = get_test_db_connection().await;

        let repo = ProjectOptionsRepository::new(db.clone());
        repo.create(ProjectOptionsActiveModel {
            id: Set(1),
            name: Set("Option A".to_owned()),
        })
        .await
        .unwrap();

        let result = repo.get_from_name("Option A").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Option A");

        clean_up_database(&db).await;
    }

    #[tokio::test]
    async fn test_get_from_id() {
        let db = get_test_db_connection().await;

        let repo = ProjectOptionsRepository::new(db.clone());
        repo.create(ProjectOptionsActiveModel {
            id: Set(1),
            name: Set("Option A".to_owned()),
        })
        .await
        .unwrap();

        let result = repo.get_from_id(1).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, 1);

        clean_up_database(&db).await;
    }

    #[tokio::test]
    async fn test_create() {
        let db = get_test_db_connection().await;

        let model = ProjectOptionsActiveModel {
            id: Set(1),
            name: Set("Option A".to_owned()),
        };

        let repo = ProjectOptionsRepository::new(db.clone());
        let result = repo.create(model).await.unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.name, "Option A");

        clean_up_database(&db).await;
    }

    #[tokio::test]
    async fn test_create_all() {
        let db = get_test_db_connection().await;

        let models = vec![
            ProjectOptionsActiveModel {
                id: Set(1),
                name: Set("Option A".to_owned()),
            },
            ProjectOptionsActiveModel {
                id: Set(2),
                name: Set("Option B".to_owned()),
            },
        ];

        let repo = ProjectOptionsRepository::new(db.clone());
        let result = repo.create_all(models).await.unwrap();
        assert_eq!(result.last_insert_id, 2);

        clean_up_database(&db).await;
    }

    #[tokio::test]
    async fn test_create_fail_constraint() {
        let db = get_test_db_connection().await;

        let model = ProjectOptionsActiveModel {
            id: NotSet,
            name: Set("Option A".to_owned()),
        };

        let repo = ProjectOptionsRepository::new(db.clone());
        let _ = repo.create(model.clone()).await.unwrap();

        let fail = repo.create(model).await;
        assert!(fail.is_err());

        assert!(matches!(fail.err().unwrap().sql_err().unwrap(), SqlErr::UniqueConstraintViolation(_)));

        clean_up_database(&db).await;
    }

    #[tokio::test]
    async fn test_update() {
        let db = get_test_db_connection().await;

        let repo = ProjectOptionsRepository::new(db.clone());
        repo.create(ProjectOptionsActiveModel {
            id: Set(1),
            name: Set("Option A".to_owned()),
        })
        .await
        .unwrap();

        let updated_model = ProjectOptionsActiveModel {
            id: Set(1),
            name: Set("Updated Option".to_owned()),
        };

        let result = repo.update(updated_model).await.unwrap();
        assert_eq!(result.name, "Updated Option");

        clean_up_database(&db).await;
    }

    #[tokio::test]
    async fn test_delete_by_id() {
        let db = get_test_db_connection().await;

        let repo = ProjectOptionsRepository::new(db.clone());
        repo.create(ProjectOptionsActiveModel {
            id: Set(1),
            name: Set("Option A".to_owned()),
        })
        .await
        .unwrap();

        let result = repo.delete_by_id(1).await.unwrap();
        assert_eq!(result.rows_affected, 1);

        clean_up_database(&db).await;
    }
}
