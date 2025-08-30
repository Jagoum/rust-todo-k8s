terraform {
  required_providers {
    multipass = {
      source = "larstobi/multipass"
      version = "1.4.2"
    }
  }
}

provider "multipass" {
}

resource "multipass_instance" "vms" {
  count = var.vm_count
  name = "machine-${count.index + 1}"
  cpus = var.vm_cpus
  memory = var.vm_memory
  disk = var.vm_disk
  cloudinit_file = "${path.module}/cloud-init.template.yaml"
}