terraform {
  required_providers {
    multipass = {
      source = "larstobi/multipass"
      version = "1.4.1"
    }
  }
}

provider "multipass" {
}

resource "multipass_instance" "control_plane" {
  name = "control-plane"
  cpus = var.control_plane_cpus
  memory = var.control_plane_memory
  disk = var.control_plane_disk
  cloud_init_file = "${path.module}/cloud-init.yaml"
}

resource "multipass_instance" "workers" {
  count = var.worker_count
  name = "worker-${count.index + 1}"
  cpus = var.worker_cpus
  memory = var.worker_memory
  disk = var.worker_disk
  cloud_init_file = "${path.module}/cloud-init.yaml"
}
