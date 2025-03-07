# cluster service account

resource "google_service_account" "employees_manager_ce" {
  account_id = "employees-manager-ce"
}

resource "google_artifact_registry_repository_iam_binding" "cluster_pull_backend" {
  repository = google_artifact_registry_repository.default_artifact_registry.name
  role       = "roles/artifactregistry.reader"
  members = [
    "serviceAccount:${google_service_account.employees_manager_ce.email}"
  ]
}