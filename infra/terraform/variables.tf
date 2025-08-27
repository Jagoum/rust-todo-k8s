variable "worker_count" {
  description = "Number of worker nodes"
  type        = number
  default     = 2
}

variable "control_plane_cpus" {
  description = "Number of CPUs for the control plane node"
  type        = number
  default     = 2
}

variable "control_plane_memory" {
  description = "Memory for the control plane node"
  type        = string
  default     = "2G"
}

variable "control_plane_disk" {
  description = "Disk size for the control plane node"
  type        = string
  default     = "10G"
}

variable "worker_cpus" {
  description = "Number of CPUs for the worker nodes"
  type        = number
  default     = 2
}

variable "worker_memory" {
  description = "Memory for the worker nodes"
  type        = string
  default     = "2G"
}

variable "worker_disk" {
  description = "Disk size for the worker nodes"
  type        = string
  default     = "10G"
}
