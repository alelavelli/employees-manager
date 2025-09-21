resource "google_service_account" "app" {
  account_id = "employees-manager-app"
}


resource "google_project_iam_binding" "secret_accessor" {
  project = var.project_id
  role    = "roles/secretmanager.secretAccessor"
  members = [
    "serviceAccount:${google_service_account.app.email}"
  ]
}
