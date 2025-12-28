# Installation Guide

Complete guide for setting up the development environment.

<toc />

## Prerequisites

Before you begin, ensure your system meets these requirements:

> [!NOTE]
> This guide assumes you're using a Unix-like operating system.
> Windows users should use WSL2 for the best experience.

### Required Software

| Software | Version       | Purpose             |
| -------- | ------------- | ------------------- |
| Node.js  | 18.x or later | Runtime environment |
| npm      | 9.x or later  | Package management  |
| Git      | 2.30+         | Version control     |
| Docker   | 24.0+         | Container runtime   |

### Optional Tools

- **VS Code** - Recommended editor with extensions
- **Postman** - API testing
- **TablePlus** - Database management

## Quick Start

<steps>
1. Clone the repository
2. Install dependencies
3. Configure environment
4. Start development server
</steps>

### Step 1: Clone the Repository

```bash
git clone https://github.com/example/project.git
cd project
```

> [!TIP]
> Use SSH for cloning if you have SSH keys configured with GitHub.
> This avoids entering credentials repeatedly.

### Step 2: Install Dependencies

```bash highlight="2" linenumbers
npm install
npm run bootstrap  # For monorepo setup
```

If you encounter permission errors:

```bash
sudo chown -R $USER ~/.npm
npm cache clean --force
npm install
```

> [!WARNING]
> Never use `sudo npm install` - fix permissions instead!

### Step 3: Configure Environment

Copy the example environment file:

```bash
cp .env.example .env
```

Edit `.env` with your configuration:

```env highlight="3-4" linenumbers
# Application
NODE_ENV=development
APP_PORT=3000
API_URL=http://localhost:3000

# Database
DB_HOST=localhost
DB_PORT=5432
DB_NAME=myapp_dev
DB_USER=postgres
DB_PASSWORD=your_password

# External Services
REDIS_URL=redis://localhost:6379
```

> [!IMPORTANT]
> Never commit `.env` files to version control!
> They contain sensitive credentials.

### Step 4: Database Setup

<tabs Database Type>
PostgreSQL
SQLite
MySQL
</tabs>

#### PostgreSQL Setup

```bash linenumbers
# Create database
createdb myapp_dev

# Run migrations
npm run db:migrate

# Seed initial data
npm run db:seed
```

#### SQLite Setup

No additional setup required. The database file will be created automatically:

```bash
npm run db:migrate
```

#### MySQL Setup

```bash
mysql -u root -p -e "CREATE DATABASE myapp_dev"
npm run db:migrate
```

### Step 5: Start Development Server

```bash
npm run dev
```

You should see:

```text
🚀 Server running at http://localhost:3000
📦 API available at http://localhost:3000/api
🔧 Admin panel at http://localhost:3000/admin
```

## Docker Setup

For containerized development:

```yaml highlight="8-12" linenumbers
# docker-compose.yml
version: "3.8"
services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=development
      - DB_HOST=db
      - REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis
```

Run with:

```bash
docker-compose up -d
```

> [!CAUTION]
> Docker Desktop on macOS has known performance issues with
> mounted volumes. Consider using docker-sync or Mutagen.

## Troubleshooting

### Common Issues

<steps>
1. **Port already in use**: Another process is using port 3000
2. **Database connection failed**: Check DB credentials
3. **Module not found**: Run `npm install` again
</steps>

#### Port Conflicts

Find and kill the process:

```bash highlight="1"
lsof -i :3000
kill -9 <PID>
```

#### Database Issues

Reset the database:

```bash linenumbers
npm run db:drop
npm run db:create
npm run db:migrate
npm run db:seed
```

> [!WARNING]
> This will delete all data! Only use in development.

#### Dependency Issues

Clear caches and reinstall:

```bash plusdiff minusdiff
- rm -rf node_modules
- rm package-lock.json
+ rm -rf node_modules package-lock.json
npm cache clean --force
npm install
```

## IDE Configuration

### VS Code Extensions

Recommended extensions for this project:

```json linenumbers
// .vscode/extensions.json
{
  "recommendations": [
    "dbaeumer.vscode-eslint",
    "esbenp.prettier-vscode",
    "prisma.prisma",
    "bradlc.vscode-tailwindcss"
  ]
}
```

### Settings

```json highlight="4-5"
// .vscode/settings.json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  }
}
```

## Next Steps

After completing setup:

1. Read the [Architecture Guide](./architecture.md)
2. Review [Contributing Guidelines](./CONTRIBUTING.md)
3. Join our [Discord Community](https://discord.gg/example)

> [!NOTE]
> Having trouble? Open an issue or ask in Discord!
