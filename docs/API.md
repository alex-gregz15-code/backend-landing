# API Reference

## Authentication

**POST /api/auth/login**
- Request JSON:

```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

- Response JSON (development stub):

```json
{
  "token": "dev-token-replace-with-jwt",
  "user": { "id": "u1", "email": "user@example.com", "role": "researcher" }
}
```

## Health / Info

**GET /api/hello**
- Response:

```json
{ "message": "Hello from BASE_APP!" }
```

## Campaigns

**GET /api/campaigns**
- Returns list of campaigns (JSON array of `Campaign`).

**POST /api/campaigns**
- Request JSON (CreateCampaign):

```json
{
  "title": "New campaign",
  "budget": 1000.0,
  "category": "Security"
}
```

- Response: `201 Created` and the newly created `Campaign` object.

**DELETE /api/campaigns/:id**
- Deletes campaign by `id`. Returns `204 No Content` on success or `404 Not Found` when missing.

## Models

`Campaign`:
- `id` (string)
- `title` (string)
- `status` (string) - e.g. `draft`, `active`
- `budget` (number)
- `category` (string)

`CreateCampaign`:
- `title` (string)
- `budget` (number)
- `category` (string)

## Notes
- Authentication is a stub in `src/main.rs` and should be replaced with real auth (JWT, DB-backed users).
- Database connection in `DATABASE_URL` should point to a Postgres instance; local docker-compose provides a sample Postgres service.

Files to inspect for implementation details: [src/main.rs](src/main.rs), [Dockerfile](Dockerfile), [docker-compose.yml](docker-compose.yml)
