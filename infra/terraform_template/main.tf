terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "6.13.0"
    }
    archive = {
      source  = "hashicorp/archive"
      version = "~> 2.2.0"
    }

  }
  backend "gcs" {
    bucket = "ml3-terraform-state"
    prefix = "employees-manager"
  }

  required_version = ">= 1.10.2"
}

provider "google" {
  project = var.project_id
  region  = var.region
}