# Todo App - Rust Backend + React Frontend

A full-stack todo application with Rust (Axum + SQLx) backend and React (Vite) frontend, containerized with multi-stage Docker builds.

## Architecture

- **Backend**: Rust with Axum web framework, SQLx for PostgreSQL, JWT authentication
- **Frontend**: React with TypeScript, Vite for fast development
- **Database**: PostgreSQL
- **Containerization**: Multi-stage Docker builds for optimized images

## Features

- User registration and authentication (JWT)
- CRUD operations for todos
- Real-time UI updates
- Responsive design
- Containerized deployment

## Development

### Prerequisites
- Rust 1.75+
- Node.js 18+
- Docker & Docker Compose
- Local registry running at `registry.local:5000`

### Backend Development
```bash
cd app
cp .env.example .env
# Edit .env with your database URL
docker-compose up -d  # Start PostgreSQL
cargo run
```

### Frontend Development
```bash
cd frontend
npm install
npm run dev
```

## Production Deployment

### Build and Test Locally
```bash
# Build all services
docker-compose build

# Run the full stack
docker-compose up -d

# Access the app
open http://localhost:3000
```

### Build and Push to Registry
```bash
# Make script executable
chmod +x build-and-push.sh

# Build, test, and push images
./build-and-push.sh v1.0.0

# Or use latest tag
./build-and-push.sh
```

### Multi-stage Build Benefits

**Backend Dockerfile:**
- Builder stage: Full Rust toolchain for compilation
- Runtime stage: Minimal Debian with only runtime dependencies
- ~1.5GB builder → ~100MB runtime image

**Frontend Dockerfile:**
- Builder stage: Node.js for building React app
- Runtime stage: Nginx serving static files
- ~1GB builder → ~50MB runtime image

## API Endpoints

### Public
- `POST /register` - User registration
- `POST /login` - User login
- `GET /healthz` - Health check

### Protected (requires JWT)
- `GET /todos` - List user's todos
- `POST /todos` - Create new todo
- `PUT /todos/:id` - Update todo
- `DELETE /todos/:id` - Delete todo

## Environment Variables

### Backend
- `DATABASE_URL` - PostgreSQL connection string
- `JWT_SECRET` - Secret key for JWT tokens
- `APP_HOST` - Server host (default: 0.0.0.0)
- `APP_PORT` - Server port (default: 8080)
- `RUST_LOG` - Log level (default: info)

### Frontend
- Proxies `/api/*` requests to backend
- No environment variables needed in production

## Testing

```bash
# Test backend
cd app && cargo test

# Test frontend
cd frontend && npm test

# Integration test with Docker
./build-and-push.sh test
```

## Registry Images

After running `build-and-push.sh`, images are available at:
- `registry.local:5000/todo-app-backend:latest`
- `registry.local:5000/todo-app-frontend:latest`
- `registry.local:5000/library/postgres:16-alpine`

## Kubernetes Deployment

Ready for Kubernetes deployment with images in your local registry. See `k8s/` directory for manifests (to be created).