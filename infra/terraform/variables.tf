variable "vm_count" {
  description = "Number of VMs"
  type        = number
  default     = 2
}

variable "vm_cpus" {
  description = "Number of CPUs for the VMs"
  type        = number
  default     = 2
}

variable "vm_memory" {
  description = "Memory for the VMs"
  type        = string
  default     = "3G"
}

variable "vm_disk" {
  description = "Disk size for the VMs"
  type        = string
  default     = "20G"
}