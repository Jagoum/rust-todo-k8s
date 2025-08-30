# Blog Backend API

A comprehensive blog backend API built with Rust, Actix Web, SQLx, and PostgreSQL. This backend provides all the functionality needed for a Medium-like blogging platform.

## Features

### 🔐 Authentication & Authorization
- User registration and login
- JWT token-based authentication
- Password hashing with bcrypt
- Protected routes with middleware

### 📝 Post Management
- Create, read, update, delete posts
- Draft and publish functionality
- Post slugs for SEO-friendly URLs
- Rich content support
- Cover image support
- Post excerpts

### 🏷️ Tagging System
- Tag posts for better organization
- Browse posts by tags
- Tag management

### 💬 Comments System
- Hierarchical commenting (replies)
- Create, update, delete comments
- Comment threading support

### 👥 Social Features
- Follow/unfollow users
- Like/unlike posts
- User profiles with bio and avatar
- Follower/following counts
- Personalized feed based on followed users

### 🔍 Content Discovery
- Paginated post listings
- User-specific drafts
- Feed for followed users
- Posts by tags

## API Endpoints

### Authentication
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - Login user
- `POST /api/v1/auth/refresh` - Refresh token

### Users
- `GET /api/v1/users/{user_id}` - Get user profile
- `GET /api/v1/users/profile` - Get current user profile
- `PUT /api/v1/users/profile` - Update profile
- `POST /api/v1/users/{user_id}/follow` - Follow user
- `DELETE /api/v1/users/{user_id}/unfollow` - Unfollow user
- `GET /api/v1/users/{user_id}/followers` - Get followers
- `GET /api/v1/users/{user_id}/following` - Get following

### Posts
- `GET /api/v1/posts` - Get published posts
- `POST /api/v1/posts` - Create new post
- `GET /api/v1/posts/{post_id}` - Get specific post
- `PUT /api/v1/posts/{post_id}` - Update post
- `DELETE /api/v1/posts/{post_id}` - Delete post
- `PATCH /api/v1/posts/{post_id}/publish` - Publish post
- `GET /api/v1/posts/drafts` - Get user's drafts
- `GET /api/v1/posts/feed` - Get personalized feed

### Comments
- `GET /api/v1/posts/{post_id}/comments` - Get post comments
- `POST /api/v1/posts/{post_id}/comments` - Create comment
- `PUT /api/v1/posts/{post_id}/comments/{comment_id}` - Update comment
- `DELETE /api/v1/posts/{post_id}/comments/{comment_id}` - Delete comment

### Likes
- `POST /api/v1/posts/{post_id}/like` - Like post
- `DELETE /api/v1/posts/{post_id}/unlike` - Unlike post

### Tags
- `GET /api/v1/tags` - Get all tags
- `GET /api/v1/tags/{tag_name}/posts` - Get posts by tag

## Tech Stack

- **Framework**: Actix Web 4.x
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT with jsonwebtoken
- **Password Hashing**: bcrypt
- **Validation**: validator crate
- **Serialization**: serde
- **Logging**: env_logger + log
- **UUID**: uuid crate
- **Date/Time**: chrono
- **Slugs**: slug crate

## Getting Started

### Prerequisites
- Rust 1.75 or later
- Docker and Docker Compose
- PostgreSQL (if running locally)

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd blog-backend
```

2. Copy environment variables:
```bash
cp .env.example .env
```

3. Update the `.env` file with your configuration.

### Running with Docker

1. Start the services:
```bash
docker-compose up -d
```

This will start:
- PostgreSQL database on port 5432
- Blog backend API on port 8080

### Running Locally

1. Install SQLx CLI:
```bash
cargo install sqlx-cli
```

2. Start PostgreSQL (using Docker):
```bash
docker run --name postgres -e POSTGRES_PASSWORD=password -e POSTGRES_DB=blog_db -p 5432:5432 -d postgres:15-alpine
```

3. Run migrations:
```bash
sqlx migrate run
```

4. Start the application:
```bash
cargo run
```

The API will be available at `http://localhost:8080`

## Database Schema

The application uses the following main tables:
- `users` - User accounts and profiles
- `posts` - Blog posts with content and metadata
- `comments` - Hierarchical comments on posts
- `likes` - User likes on posts
- `follows` - User follow relationships
- `tags` - Post tags for categorization
- `post_tags` - Junction table for post-tag relationships

## Authentication

The API uses JWT tokens for authentication. Include the token in the Authorization header:

```
Authorization: Bearer <your-jwt-token>
```

### Registration Example
```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "email": "john@example.com",
    "password": "securepassword123",
    "full_name": "John Doe"
  }'
```

### Login Example
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "password": "securepassword123"
  }'
```

## Creating a Post Example
```bash
curl -X POST http://localhost:8080/api/v1/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-token>" \
  -d '{
    "title": "My First Blog Post",
    "content": "This is the content of my first blog post...",
    "excerpt": "A short description of the post",
    "tags": ["technology", "rust", "web-development"]
  }'
```

## Response Format

All API responses follow this structure:

```json
{
  "success": true,
  "data": { ... },
  "message": null
}
```

For errors:
```json
{
  "success": false,
  "data": null,
  "message": "Error description"
}
```

## Pagination

List endpoints support pagination with query parameters:
- `page` - Page number (default: 1)
- `limit` - Items per page (default: 20)

Example: `GET /api/v1/posts?page=2&limit=10`

## Development

### Database Migrations

Create a new migration:
```bash
sqlx migrate add <migration_name>
```

Run migrations:
```bash
sqlx migrate run
```

### Testing

Run tests:
```bash
cargo test
```

## Production Deployment

1. Set strong JWT secret in production
2. Use environment variables for sensitive configuration
3. Enable HTTPS
4. Set up proper logging and monitoring
5. Use a production-grade PostgreSQL setup
6. Consider using a reverse proxy like nginx

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is open source and available under the [MIT License](LICENSE).

## Future Enhancements

- [ ] Image upload for posts and avatars
- [ ] Email verification
- [ ] Password reset functionality
- [ ] Post bookmarks/saved posts
- [ ] Search functionality
- [ ] Admin panel
- [ ] Content moderation
- [ ] Rate limiting
- [ ] Caching layer
- [ ] Real-time notifications
- [ ] Import/export functionality