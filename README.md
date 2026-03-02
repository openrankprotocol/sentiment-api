# Sentiment API

A Rust-based API for Vercel Serverless Functions that provides sentiment analysis based on weighted post data.

## Endpoints

### 1. GET `/api/topics`

Returns all available topics for a given category.

**Query Parameters:**
- `category` (required): The category to filter topics by

**Example:**
```
GET /api/topics?category=politics
```

**Response:**
```json
[
  {
    "id": 1,
    "name": "2024 Election",
    "category": "politics"
  },
  {
    "id": 2,
    "name": "Climate Policy",
    "category": "politics"
  }
]
```

### 2. GET `/api/posts`

Returns top posts for a topic, sorted by `weight * relevancy`.

**Query Parameters:**
- `topic_id` (required): The ID of the topic
- `limit` (optional): Maximum number of posts to return (default: 10)

**Example:**
```
GET /api/posts?topic_id=1&limit=5
```

**Response:**
```json
[
  {
    "id": 1,
    "topic_id": 1,
    "content": "Post content here...",
    "weight": 0.9,
    "relevancy": 0.85,
    "score": 0.765
  }
]
```

### 3. GET `/api/sentiment`

Returns normalized sentiment probabilities for all outcomes of a topic.

The sentiment is calculated by:
1. For each post and outcome: `weight * relevancy * probability`
2. Sum all scores for each outcome
3. Normalize probabilities so they sum to 1.0

**Query Parameters:**
- `topic_id` (required): The ID of the topic

**Example:**
```
GET /api/sentiment?topic_id=1
```

**Response:**
```json
{
  "topic_id": 1,
  "topic_name": "2024 Election",
  "sentiments": [
    {
      "outcome_id": 1,
      "outcome_name": "Candidate A Wins",
      "probability": 0.55
    },
    {
      "outcome_id": 2,
      "outcome_name": "Candidate B Wins",
      "probability": 0.45
    }
  ]
}
```

## Environment Variables

Create a `.env` file with the following variables:

```
DATABASE_URL=postgres://user:password@host:port/database
```

## Database Schema

The API expects the following PostgreSQL tables:

```sql
CREATE TABLE topics (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    category VARCHAR(255) NOT NULL
);

CREATE TABLE posts (
    id BIGSERIAL PRIMARY KEY,
    topic_id BIGINT NOT NULL REFERENCES topics(id),
    content TEXT NOT NULL,
    weight DOUBLE PRECISION NOT NULL,
    relevancy DOUBLE PRECISION NOT NULL
);

CREATE TABLE outcomes (
    id BIGSERIAL PRIMARY KEY,
    topic_id BIGINT NOT NULL REFERENCES topics(id),
    name VARCHAR(255) NOT NULL
);

CREATE TABLE post_outcome_probabilities (
    id BIGSERIAL PRIMARY KEY,
    post_id BIGINT NOT NULL REFERENCES posts(id),
    outcome_id BIGINT NOT NULL REFERENCES outcomes(id),
    probability DOUBLE PRECISION NOT NULL
);

CREATE INDEX idx_topics_category ON topics(category);
CREATE INDEX idx_posts_topic_id ON posts(topic_id);
CREATE INDEX idx_outcomes_topic_id ON outcomes(topic_id);
CREATE INDEX idx_post_outcome_probabilities_post_id ON post_outcome_probabilities(post_id);
```

## Development

### Prerequisites

- Rust (latest stable)
- PostgreSQL database

### Local Development

1. Install dependencies:
   ```bash
   cargo build
   ```

2. Set up your `.env` file with database credentials

3. Run locally with Vercel CLI:
   ```bash
   vercel dev
   ```

### Deployment

Deploy to Vercel:

```bash
vercel
```

Make sure to set the `DATABASE_URL` environment variable in your Vercel project settings.

## Project Structure

```
sentiment-api/
├── api/
│   ├── topics.rs      # Topics endpoint handler
│   ├── posts.rs       # Posts endpoint handler
│   └── sentiment.rs   # Sentiment endpoint handler
├── src/
│   └── lib.rs         # Shared database models and queries
├── Cargo.toml         # Rust dependencies
├── vercel.json        # Vercel configuration
└── README.md
```
