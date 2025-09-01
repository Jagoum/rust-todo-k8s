## Plan for "Forge Your Application" Phase

**Project Goal:** Develop a basic blog post application similar to Medium, supporting multiple users, posts, comments, and likes, with JWT authentication. The application will be modular and designed to integrate with external image services.

**Technologies:**
*   **Rust Framework:** Actix Web
*   **ORM:** Diesel
*   **Database:** PostgreSQL (as implied by the PDF)

**High-Level Plan:**

1.  **Setup & Modularity:** Configure `Cargo.toml`, organize code into `models`, `handlers`, `db`, and `auth` modules.
2.  **Database Schema:** Design and implement Diesel migrations for `users`, `posts`, `comments`, and `likes` tables.
3.  **User Authentication:** Implement user registration, login, and JWT token generation/validation.
4.  **Blog Post Management:** Implement CRUD operations for blog posts, associating them with users.
5.  **User Interactions:** Implement functionality for users to add comments and likes to posts.
6.  **Image Integration (Placeholder):** Design the database to store image URLs, acknowledging that actual image storage will be handled by a separate service.

**Detailed Plan:**

**Phase 1: Project Setup and Database Schema**
*   **1.1 `Cargo.toml`:** Add `actix-web`, `diesel` (with `postgres` feature), `dotenv`, `serde`, `chrono`, `uuid`, `argon2` (for password hashing), and `jsonwebtoken` dependencies.
*   **1.2 Modularity:** Create `src/models.rs`, `src/schema.rs`, `src/db.rs`, `src/handlers.rs`, and `src/auth.rs`.
*   **1.3 Diesel Migrations:**
    *   Initialize Diesel: `diesel setup`.
    *   Create migrations for:
        *   `users`: `id`, `username`, `email`, `password_hash`, `created_at`, `updated_at`.
        *   `posts`: `id`, `user_id` (foreign key to users), `title`, `content`, `image_url` (optional), `created_at`, `updated_at`.
        *   `comments`: `id`, `user_id` (foreign key to users), `post_id` (foreign key to posts), `content`, `created_at`, `updated_at`.
        *   `likes`: `id`, `user_id` (foreign key to users), `post_id` (foreign key to posts), `created_at` (unique constraint on `user_id` and `post_id`).
    *   Run migrations: `diesel migration run`.

**Phase 2: User Authentication and Management**
*   **2.1 User Model:** Define `User` struct in `src/models.rs` with `Queryable` and `Insertable` traits.
*   **2.2 Password Hashing:** Implement `argon2` for hashing passwords during registration and verifying during login.
*   **2.3 JWT:** Implement JWT token generation upon successful login and create an Actix Web extractor/middleware for validating tokens on protected routes.
*   **2.4 Handlers:** Create `register`, `login`, and `me` (protected) handlers in `src/handlers.rs`.

**Phase 3: Blog Post Management**
*   **3.1 Post Model:** Define `Post` struct in `src/models.rs`.
*   **3.2 Handlers:** Implement `create_post` (protected), `get_posts`, `get_post_by_id`, `update_post` (protected, owner only), and `delete_post` (protected, owner only) handlers in `src/handlers.rs`.

**Phase 4: User Interactions (Likes & Comments)**
*   **4.1 Models:** Define `Comment` and `Like` structs in `src/models.rs`.
*   **4.2 Handlers:** Implement `add_comment` (protected), `get_comments_for_post`, `like_post` (protected), and `unlike_post` (protected) handlers in `src/handlers.rs`.

**Phase 5: Integration and Refinement**
*   **5.1 Database Pooling:** Set up `r2d2` or `deadpool` for efficient database connection management in `src/db.rs`.
*   **5.2 Error Handling:** Implement consistent error handling for all API endpoints.
*   **5.3 Routing:** Define all API routes in `src/main.rs`.
*   **5.4 Image URLs:** Ensure the `image_url` field in the `Post` model correctly handles optional string values. (Note: Actual image upload/storage will be external to this application's scope for this phase).

**Important Considerations:**
*   **Keycloak:** While the PDF mentions Keycloak, for this phase, the Rust application will handle JWT validation internally. Full Keycloak integration would be a separate, more complex step.
*   **Image Storage:** This plan focuses on storing image *URLs*. The actual image files would reside in a separate service (e.g., S3, MinIO, or a custom container) that the application would reference.

This plan provides a solid foundation for a basic blog post application, covering essential functional requirements. Additional features can be built upon this core in future iterations.