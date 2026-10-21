# Rust CEX

A small full-stack centralized exchange demo built with Rust and Actix Web. It provides a SOL/USD limit order book, JWT authentication, in-memory balances, and a browser UI served by the Rust backend.

## Features

- Account signup and JWT-based signin
- Demo USD on-ramp
- Demo SOL deposits
- SOL/USD limit orders
- Price-time-priority order matching
- Partial fills and resting orders
- Available and locked USD/SOL balances
- Open-order listing and owner-only cancellation
- Embedded browser dashboard

The application uses in-memory state for simplicity. Accounts, balances, and orders reset when the server restarts.

## Requirements

- Rust stable toolchain
- Cargo
- Windows users: Visual Studio Build Tools with the **Desktop development with C++** workload and a Windows SDK

Install Rust on Windows with:

```powershell
winget install --id Rustlang.Rustup
```

Install the Microsoft C++ build tools with:

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools
```

When the Visual Studio Installer opens, select **Desktop development with C++**.

## Run

From the repository root:

```powershell
cargo run
```

The server listens on:

```text
http://127.0.0.1:3001
```

Open that URL in a browser to use the dashboard.

On Windows, if Cargo cannot find `link.exe`, launch **Developer PowerShell for VS 2022** before running `cargo run`.

## API

Authenticated endpoints use:

```text
Authorization: Bearer <token>
```

| Method | Endpoint | Authentication | Description |
| --- | --- | --- | --- |
| `POST` | `/signup` | No | Create an account |
| `POST` | `/signin` | No | Receive a JWT |
| `GET` | `/balance` | Yes | Read USD and SOL balances |
| `POST` | `/onramp` | Yes | Add demo USD |
| `POST` | `/deposit/sol` | Yes | Add demo SOL |
| `GET` | `/orderbook` | No | Read current SOL/USD levels |
| `GET` | `/orders` | Yes | Read the current user's open orders |
| `POST` | `/order` | Yes | Place a SOL/USD limit order |
| `POST` | `/cancel` | Yes | Cancel an open order |

### Example requests

Create an account:

```powershell
Invoke-RestMethod -Method Post `
  -Uri http://127.0.0.1:3001/signup `
  -ContentType "application/json" `
  -Body '{"username":"trader","password":"secret123"}'
```

Sign in:

```powershell
$signin = Invoke-RestMethod -Method Post `
  -Uri http://127.0.0.1:3001/signin `
  -ContentType "application/json" `
  -Body '{"username":"trader","password":"secret123"}'

$headers = @{ Authorization = "Bearer $($signin.token)" }
```

Add demo USD:

```powershell
Invoke-RestMethod -Method Post `
  -Uri http://127.0.0.1:3001/onramp `
  -Headers $headers `
  -ContentType "application/json" `
  -Body '{"qty":1000}'
```

Place a buy order:

```powershell
Invoke-RestMethod -Method Post `
  -Uri http://127.0.0.1:3001/order `
  -Headers $headers `
  -ContentType "application/json" `
  -Body '{"side":"bid","asset":"sol","price":150,"qty":1}'
```

## Project structure

```text
.
├── Cargo.toml
└── src/
    ├── main.rs             # Actix server and application state
    ├── orderbook.rs        # Matching engine and order book
    ├── middleware/
    │   └── mod.rs          # JWT authentication extractor
    ├── routes/
    │   └── user.rs         # Authentication, balances, and trading API
    ├── types/
    │   └── user.rs         # Request, response, and domain types
    └── public/
        └── index.html      # Embedded browser dashboard
```

## Development checks

Run the compiler checks with:

```powershell
cargo check
```

Format the code with:

```powershell
cargo fmt
```
