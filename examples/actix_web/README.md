# Actix Web + AuthBox Example

## Features

- User Registration
- User Login
- JWT Authentication
- Access Token Validation
- Refresh Tokens
- Email Verification
- Forgot Password
- Reset Password
- Protected Route (`/api/me`)

---

## Prerequisites

- Rust (latest stable)
- Cargo

Verify installation:

```bash
rustc --version
cargo --version
```

---

## Running the Application

Start the server:

```bash
cargo run
```

The application will be available at:

```text
http://127.0.0.1:8080
```

---

## Available Routes

| Method | Route | Description |
|--------|--------|-------------|
| POST | `/register` | Register a new user |
| POST | `/login` | Authenticate user |
| POST | `/refresh` | Refresh access token |
| POST | `/verify-email` | Verify email address |
| POST | `/forgot-password` | Request password reset |
| POST | `/reset-password` | Reset password |
| GET | `/api/me` | Get current authenticated user |

---

## cURL Examples

### Register

```bash
curl -X POST http://127.0.0.1:8080/register \
-H "Content-Type: application/json" \
-d '{
  "email":"john@test.com",
  "password":"password123",
  "username":"john",
  "phone":null,
  "country":"Kenya",
  "city":"Nairobi",
  "age":25
}'
```

After registering, check your server logs.

The application prints verification tokens and password reset tokens there for development purposes. Copy these tokens manually from the logs to proceed with email verification or password reset.

---

### Login

```bash
curl -X POST http://127.0.0.1:8080/login \
-H "Content-Type: application/json" \
-d '{
  "email":"john@test.com",
  "password":"password123"
}'
```

#### Response

```json
{
  "access": "eyJ...",
  "refresh": "eyJ..."
}
```

---

### Get Current User

```bash
curl http://127.0.0.1:8080/api/me \
-H "Authorization: Bearer ACCESS_TOKEN"
```

---

### Refresh Token

```bash
curl -X POST http://127.0.0.1:8080/refresh \
-H "Content-Type: application/json" \
-d '{
  "refresh_token":"REFRESH_TOKEN"
}'
```

---

### Verify Email

After registration, check the server logs for the email verification token, then run:

```bash
curl -X POST http://127.0.0.1:8080/verify-email \
-H "Content-Type: application/json" \
-d '{
  "token":"EMAIL_VERIFICATION_TOKEN_FROM_LOGS"
}'
```

---

### Forgot Password

```bash
curl -X POST http://127.0.0.1:8080/forgot-password \
-H "Content-Type: application/json" \
-d '{
  "email":"john@test.com"
}'
```

Check the server logs for the password reset token.

---

### Reset Password

Use the token printed in the server logs:

```bash
curl -X POST http://127.0.0.1:8080/reset-password \
-H "Content-Type: application/json" \
-d '{
  "token":"RESET_TOKEN_FROM_LOGS",
  "password":"new-password"
}'
```
