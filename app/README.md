Todo App (Axum + SQLx + Postgres)

Overview
- Backend: Axum 0.7
- DB: SQLx 0.7 (Postgres)
- Auth: Argon2 password hashing + JWT (HS256)
- Endpoints:
  - POST /register {username, password}
  - POST /login {username, password} -> {token}
  - GET /todos (auth)
  - POST /todos {title} (auth)
  - PUT /todos/:id {title?, completed?} (auth)
  - DELETE /todos/:id (auth)

Environment
- Copy .env.example to .env and set:
  - DATABASE_URL=postgres://todo_user:todo_password@localhost:5432/todo_db
  - JWT_SECRET=change_me
  - APP_HOST=0.0.0.0
  - APP_PORT=8080

Local Postgres via local registry
- Assumes local registry is available at registry.local:5000 as configured by infra role.
- Example docker-compose.yml will pull registry.local:5000/library/postgres:16-alpine.

Run locally
- cargo run
- Or use docker-compose to launch db then run the app.

