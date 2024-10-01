# DevGrowth

DevGrowth is a Rust-based web application that interacts with GitHub repositories, allowing users to analyze developer growth through the Growth Accounting framework.

## Prerequisites

- Rust (latest stable version)
- PostgreSQL
- GitHub Personal Access Token

## Setup

1. Clone the repository:
   ```
   git clone https://github.com/Martian-Engineering/devgrowth.git
   cd devgrowth
   ```

2. Set up the database:
   - Create a PostgreSQL database named `devgrowth`
   - Run the SQL migrations:
     ```
     sqlx migrate run
     ```

3. Set up environment variables:
   Create a `.env` file in the project root with the following content:
   ```
   DB_USER=username
   DB_PASS=password
   GITHUB_TOKEN=your_github_personal_access_token
   ```
   Replace `username`, `password`, and `your_github_personal_access_token` with your actual values.

4. Build the project:
   ```
   cargo build
   ```

## Running the Application

To run the application:

```
cargo run
```

The server will start at `http://localhost:8080`.

## API Endpoints

- `GET /`: Hello world endpoint
- `POST /repositories`: Create a new repository
- `PUT /repositories/{owner}/{name}`: Sync a repository
- `GET /repositories/{owner}/{name}`: Get repository metadata

## Testing

To run the tests:

```
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
