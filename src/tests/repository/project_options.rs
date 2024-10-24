#[cfg(test)]
mod tests {
    use entity::project_options;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Transaction};
    use sea_orm::prelude::*;
    use entity::project_options::{Model as ProjectOptionsModel, ActiveModel as ProjectOptionsActiveModel};

    #[tokio::test]
    async fn test_get_all() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                vec![
                    project_options::Model {
                        id: 1,
                        name: "Option A".to_owned(),
                        value: "Value A".to_owned(),
                    },
                    project_options::Model {
                        id: 2,
                        name: "Option B".to_owned(),
                        value: "Value B".to_owned(),
                    },
                ]
            ])
            .into_connection();

        let repo = ProjectOptionsRepository::new(mock_db);
        let result = repo.get_all().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Option A");
        assert_eq!(result[1].name, "Option B");
    }

    #[tokio::test]
    async fn test_get_from_name() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                vec![
                    project_options::Model {
                        id: 1,
                        name: "Option A".to_owned(),
                        value: "Value A".to_owned(),
                    }
                ]
            ])
            .into_connection();

        let repo = ProjectOptionsRepository::new(mock_db);
        let result = repo.get_from_name("Option A").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Option A");
    }

    #[tokio::test]
    async fn test_get_from_id() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                vec![
                    project_options::Model {
                        id: 1,
                        name: "Option A".to_owned(),
                        value: "Value A".to_owned(),
                    }
                ]
            ])
            .into_connection();

        let repo = ProjectOptionsRepository::new(mock_db);
        let result = repo.get_from_id(1).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, 1);
    }

    #[tokio::test]
    async fn test_create() {
        let model = ProjectOptionsActiveModel {
            id: Set(1),
            name: Set("Option A".to_owned()),
            value: Set("Value A".to_owned()),
        };

        let mock_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .append_query_results(vec![
                vec![
                    project_options::Model {
                        id: 1,
                        name: "Option A".to_owned(),
                        value: "Value A".to_owned(),
                    }
                ]
            ])
            .into_connection();

        let repo = ProjectOptionsRepository::new(mock_db);
        let result = repo.create(model).await.unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.name, "Option A");
    }

    #[tokio::test]
    async fn test_create_all() {
        let models = vec![
            ProjectOptionsActiveModel {
                id: Set(1),
                name: Set("Option A".to_owned()),
                value: Set("Value A".to_owned()),
            },
            ProjectOptionsActiveModel {
                id: Set(2),
                name: Set("Option B".to_owned()),
                value: Set("Value B".to_owned()),
            },
        ];

        let mock_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 2,
            }])
            .into_connection();

        let repo = ProjectOptionsRepository::new(mock_db);
        let result = repo.create_all(models).await.unwrap();
        assert_eq!(result.rows_affected, 2);
    }

    #[tokio::test]
    async fn test_update() {
        let model = ProjectOptionsActiveModel {
            id: Set(1),
            name: Set("Updated Option".to_owned()),
            value: Set("Updated Value".to_owned()),
        };

        let mock_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results(vec![
                vec![
                    project_options::Model {
                        id: 1,
                        name: "Updated Option".to_owned(),
                        value: "Updated Value".to_owned(),
                    }
                ]
            ])
            .into_connection();

        let repo = ProjectOptionsRepository::new(mock_db);
        let result = repo.update(model).await.unwrap();
        assert_eq!(result.name, "Updated Option");
    }

    #[tokio::test]
    async fn test_delete_by_id() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();

        let repo = ProjectOptionsRepository::new(mock_db);
        let result = repo.delete_by_id(1).await.unwrap();
        assert_eq!(result.rows_affected, 1);
    }
}
