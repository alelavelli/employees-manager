resource "google_compute_instance" "default" {
  name         = "employees-manager"
  machine_type = "e2-medium"
  zone         = var.zone
  tags         = [var.network_tag]

  boot_disk {
    auto_delete = true
    initialize_params {
      image = "projects/ubuntu-os-cloud/global/images/ubuntu-2404-noble-amd64-v20250228"
      size  = 32
      type  = "pd-balanced"
    }

    mode = "READ_WRITE"
  }

  network_interface {
    network = google_compute_network.default.name
    subnetwork = google_compute_subnetwork.default.name
    access_config {
      network_tier = "STANDARD"
      
    }
  }

  service_account {
    email  = google_service_account.employees_manager_ce.email
    scopes = ["cloud-platform"]
  }
}