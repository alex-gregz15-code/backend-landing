# Deployment Guide

## Deploying to Render

### Prerequisites
- GitHub repo with this code
- Render account (free tier available)
- PostgreSQL database (use Render's Postgres or Supabase)

### Step 1: Set up Database

**Option A: Render PostgreSQL**
- Go to [render.com](https://render.com)
- Create a new PostgreSQL instance
- Note the internal connection URL (use inside private network) or external URL
- Copy the connection string

**Option B: Supabase**
- Use the existing Supabase `DATABASE_URL` from your `.env`

### Step 2: Create Web Service on Render

1. Go to **Dashboard** → **New+** → **Web Service**
2. Connect your GitHub repo (base_app)
3. Fill in:
   - **Name**: `base-app-api` (or your choice)
   - **Environment**: `Rust`
   - **Build Command**: `cargo build --release --bin base_app`
   - **Start Command**: `/app/target/release/base_app`
   - **Plan**: Free or Starter

### Step 3: Add Environment Variables

In the Render dashboard, go to **Environment** and add:

```
DATABASE_URL=postgresql://user:password@host:5432/dbname
FRONTEND_URL=https://your-frontend.vercel.app
SQLX_OFFLINE=true
```

**Critical:** Set `FRONTEND_URL` to your deployed frontend domain (or `http://localhost:5173` for local testing only).

### Step 4: Deploy

- Click **Create Web Service**
- Render auto-deploys on push to main branch
- Check the **Logs** tab if deployment fails

### Troubleshooting

**"Application exited early"**
- Check logs for the actual error (should show more details now with better error handling)
- Verify `FRONTEND_URL` is set correctly and valid
- Verify `DATABASE_URL` is correct and reachable
- Ensure `PORT` is not hardcoded (Render assigns it via `$PORT` env var)

**"Connection refused" to database**
- Make sure your PostgreSQL is in the same private network or has internet access
- If using external DB (Supabase), check network firewall rules
- Test connection locally first: `psql $DATABASE_URL -c "SELECT 1;"`

**Build fails with "rs: not found"**
- Already fixed in `Dockerfile` (check line 22 uses `rm -rf` not `rs -rf`)

### Health Check

The backend exposes `/api/hello` which Render uses for health checks. If you see restart loops, check this endpoint locally:

```bash
curl http://localhost:8000/api/hello
```

Expected response:
```json
{"message":"Hello from BASE_APP!"}
```

### Local Testing Before Deploy

```bash
# Set env vars locally
export DATABASE_URL="your-db-connection"
export FRONTEND_URL="http://localhost:5173"

# Build and run
cargo build --release
./target/release/base_app
```

Then test:
```bash
curl http://localhost:8000/api/hello
curl http://localhost:8000/api/campaigns
```

---

See [../README.md](../README.md) for local development and [API.md](API.md) for endpoint details.
