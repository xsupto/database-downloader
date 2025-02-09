# Rust Backup Utility with Tokio Scheduler (Kubernetes Deployment)

This project is a command-line utility written in Rust that automates database backups and manages old backup files. It leverages the power of Rust, `Tokio` for asynchronous operations, and the `tokio_scheduler` crate for scheduled execution. It also uses the `sh` crate for running shell commands.  This version includes instructions for Dockerization and Kubernetes deployment.

## Features

* **Database Backup:** Performs database backups using a configurable shell command.
* **Scheduled Backups:** Schedules backups at specified intervals using `tokio_scheduler`.
* **Old Backup Cleanup:** Automatically deletes old backups based on a configurable retention policy.
* **Asynchronous Operations:** Utilize `Tokio` for efficient and non-blocking operations.
* **Shell Command Execution:** Employs the `sh` crate for executing shell commands, providing flexibility in backup and cleanup procedures.
* **Logging:** Includes logging to track backup and cleanup operations.
* **Dockerized:** Easily packaged and deployed as a Docker container.
* **Kubernetes Deployment:**  Supports deployment within a Kubernetes cluster.

## Technologies Used

* **Rust:** The primary programming language.
* **Tokio:** Asynchronous runtime for Rust.
* **tokio_scheduler:** For scheduling backup tasks.
* **sh:** For executing shell commands.
* **Docker:** For containerization.
* **Kubernetes:** For container orchestration.

## Installation (Building the Binary)

```bash
git clone https://github.com/xsupto/database-downloader.git  
cargo build --release