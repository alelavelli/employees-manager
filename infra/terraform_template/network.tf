resource "google_compute_network" "default" {
  name = "employees-manager"

  auto_create_subnetworks  = false
  enable_ula_internal_ipv6 = true
}

resource "google_compute_subnetwork" "default" {
  name = "employees-manager-subnet"

  ip_cidr_range = "10.128.0.0/20"
  region        = var.region

  stack_type       = "IPV4_IPV6"
  ipv6_access_type = "INTERNAL"

  network = google_compute_network.default.id
}

resource "google_compute_firewall" "default" {
  name    = "employees-manager-firewall"
  network = google_compute_network.default.name
  source_ranges = ["0.0.0.0/0"]
  allow {
    protocol = "tcp"
    ports    = ["22", "80", "443"]
  }

  allow {
    protocol = "icmp"
  }

  target_tags = [var.network_tag]
}
