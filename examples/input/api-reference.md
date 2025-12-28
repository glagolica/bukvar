# API Reference

Complete API documentation for the REST endpoints.

<toc />

## Overview

Base URL: `https://api.example.com/v1`

All requests must include authentication headers.

> [!IMPORTANT]
> API keys should be kept secret and never exposed in client-side code.

## Authentication

### API Key Authentication

Include your API key in the request header:

```http
Authorization: Bearer your_api_key_here
```

### OAuth 2.0

For user-scoped operations, use OAuth 2.0:

```javascript linenumbers
const response = await fetch("/oauth/token", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    grant_type: "authorization_code",
    code: authCode,
    redirect_uri: "https://yourapp.com/callback",
  }),
});
```

> [!NOTE]
> Access tokens expire after 1 hour. Use refresh tokens for long-lived sessions.

## Users

### List Users

```http
GET /users
```

**Query Parameters:**

| Parameter | Type    | Description                            |
| --------- | ------- | -------------------------------------- |
| `page`    | integer | Page number (default: 1)               |
| `limit`   | integer | Items per page (default: 20, max: 100) |
| `sort`    | string  | Sort field (created_at, name, email)   |
| `order`   | string  | Sort order (asc, desc)                 |

**Response:**

```json highlight="3-8" linenumbers
{
  "data": [
    {
      "id": "usr_123abc",
      "email": "user@example.com",
      "name": "John Doe",
      "created_at": "2024-01-15T10:30:00Z"
    }
  ],
  "meta": {
    "total": 150,
    "page": 1,
    "limit": 20
  }
}
```

### Get User

```http
GET /users/{id}
```

**Path Parameters:**

| Parameter | Type   | Description |
| --------- | ------ | ----------- |
| `id`      | string | User ID     |

**Response:**

```json linenumbers
{
  "id": "usr_123abc",
  "email": "user@example.com",
  "name": "John Doe",
  "role": "admin",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-20T15:45:00Z"
}
```

> [!TIP]
> Use the `expand` query parameter to include related resources:
> `GET /users/usr_123?expand=orders,addresses`

### Create User

```http
POST /users
```

**Request Body:**

```json linenumbers
{
  "email": "newuser@example.com",
  "name": "Jane Smith",
  "password": "secure_password_123",
  "role": "member"
}
```

**Response:** `201 Created`

```json
{
  "id": "usr_456def",
  "email": "newuser@example.com",
  "name": "Jane Smith",
  "role": "member",
  "created_at": "2024-01-25T08:00:00Z"
}
```

> [!WARNING]
> Passwords must be at least 12 characters and include:
>
> - Uppercase and lowercase letters
> - At least one number
> - At least one special character

### Update User

```http
PATCH /users/{id}
```

**Request Body:**

```json
{
  "name": "Jane Doe",
  "role": "admin"
}
```

> [!CAUTION]
> Changing a user's role may affect their access to resources.
> This action is logged for audit purposes.

### Delete User

```http
DELETE /users/{id}
```

**Response:** `204 No Content`

## Orders

### Order Lifecycle

<steps>
1. Order created with `pending` status
2. Payment processed, status becomes `paid`
3. Items shipped, status becomes `shipped`
4. Customer receives items, status becomes `delivered`
</steps>

### Create Order

```http
POST /orders
```

**Request Body:**

```json highlight="4-12" linenumbers
{
  "user_id": "usr_123abc",
  "currency": "USD",
  "items": [
    {
      "product_id": "prod_abc123",
      "quantity": 2,
      "unit_price": 1999
    },
    {
      "product_id": "prod_def456",
      "quantity": 1,
      "unit_price": 4999
    }
  ],
  "shipping_address": {
    "line1": "123 Main St",
    "city": "San Francisco",
    "state": "CA",
    "postal_code": "94102",
    "country": "US"
  }
}
```

> [!NOTE]
> Prices are in cents (smallest currency unit).
> $19.99 = 1999 cents.

#### Pending → Paid

```http
POST /orders/{id}/pay
```

```json
{
  "payment_method_id": "pm_card_visa",
  "amount": 8997
}
```

#### Paid → Shipped

```http
POST /orders/{id}/ship
```

```json
{
  "carrier": "ups",
  "tracking_number": "1Z999AA10123456784"
}
```

#### Cancel Order

```http
POST /orders/{id}/cancel
```

```json
{
  "reason": "customer_request",
  "refund": true
}
```

## Webhooks

### Setting Up Webhooks

Register a webhook endpoint to receive real-time events:

```http
POST /webhooks
```

```json linenumbers
{
  "url": "https://yourapp.com/webhooks/handler",
  "events": ["order.created", "order.paid", "order.shipped", "user.created"],
  "secret": "whsec_your_webhook_secret"
}
```

### Webhook Payload

```json highlight="2,7-10" linenumbers
{
  "id": "evt_1234567890",
  "type": "order.paid",
  "created_at": "2024-01-25T10:30:00Z",
  "data": {
    "object": {
      "id": "ord_abc123",
      "status": "paid",
      "amount": 8997,
      "currency": "USD"
    }
  }
}
```

### Verifying Signatures

Always verify webhook signatures:

```javascript highlight="5-8" linenumbers
const crypto = require("crypto");

function verifyWebhook(payload, signature, secret) {
  const expected = crypto
    .createHmac("sha256", secret)
    .update(payload)
    .digest("hex");
  return crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expected));
}
```

> [!IMPORTANT]
> Always use constant-time comparison (`timingSafeEqual`) to prevent
> timing attacks when verifying signatures.

## Error Handling

### Error Response Format

```json linenumbers
{
  "error": {
    "code": "invalid_request",
    "message": "The request body is missing required field: email",
    "details": [
      {
        "field": "email",
        "issue": "required"
      }
    ],
    "request_id": "req_abc123xyz"
  }
}
```

### Error Codes

| Code                    | HTTP Status | Description              |
| ----------------------- | ----------- | ------------------------ |
| `invalid_request`       | 400         | Malformed request        |
| `authentication_failed` | 401         | Invalid credentials      |
| `permission_denied`     | 403         | Insufficient permissions |
| `not_found`             | 404         | Resource doesn't exist   |
| `rate_limited`          | 429         | Too many requests        |
| `internal_error`        | 500         | Server error             |

## Rate Limiting

API requests are rate limited per API key:

| Plan       | Requests/minute | Requests/day |
| ---------- | --------------- | ------------ |
| Free       | 60              | 1,000        |
| Pro        | 600             | 50,000       |
| Enterprise | 6,000           | Unlimited    |

Rate limit headers:

```http
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1706180400
```

> [!TIP]
> Implement exponential backoff when you receive a 429 response.
> Start with a 1-second delay and double it on each retry.
