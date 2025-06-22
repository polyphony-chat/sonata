[![Coverage Status](https://coveralls.io/repos/github/polyphony-chat/sonata/badge.svg?branch=main)](https://coveralls.io/github/polyphony-chat/sonata?branch=main)
[![FAQ-shield]][FAQ]
[![Discord]][Discord-invite]
<img src="https://img.shields.io/static/v1?label=Status&message=Early%20Development&color=blue">

# sonata

A robust, performant implementation of a polyproto home server.

> [!IMPORTANT]
>
> What is the difference between this project and [symfonia](https://github.com/polyphony-chat/symfonia)?

> [!TIP]
>
> sonata is a standalone polyproto home server, taking care of all the routes and behaviors defined in the polyproto-core specification.
>
> Symfonia is a polyproto-chat server, exclusively caring about the polyproto-chat extension and the routes and behaviors defined by *it*.

---

> [!NOTE]
> This software is not yet ready to be used.

## About

sonata is a robust, performant polyproto home server implementation built with Rust. It provides a complete implementation of the polyproto-core specification, handling all the routes and behaviors defined therein.

## Development Setup

### Prerequisites

Before setting up the development environment, you'll need to install the following dependencies:

- **Rust**: Install Rust and Cargo from [https://rustup.rs/](https://rustup.rs/)
- **PostgreSQL**: Install PostgreSQL from [https://www.postgresql.org/download/](https://www.postgresql.org/download/)
- **pre-commit**: Install pre-commit from [https://pre-commit.com/](https://pre-commit.com/)

### Environment Configuration

1. Copy the example environment file:

   ```bash
   cp .example.env .env
   ```

2. Edit the `.env` file with your database credentials and configuration.

### Database Setup

1. Create a PostgreSQL database for sonata:

   ```sql
   CREATE DATABASE sonata;
   CREATE USER sonata WITH PASSWORD 'sonata';
   GRANT ALL PRIVILEGES ON DATABASE sonata TO sonata;
   ```

2. The database migrations will be automatically applied when you run the application.

### Building and Running

#### Release Builds

sonata is configured with aggressive optimizations for production releases. This significantly reduces binary size while maintaining performance.

To build sonata for release:

```bash
cargo build --release --config .config/release.toml
```

This will create an optimized executable in `target/release/sonata`.

For development and testing, use the standard debug build:

```bash
cargo build
```

### Development Tools

#### Pre-commit Hooks

This project uses pre-commit hooks to ensure code quality. Install and set up the hooks:

```bash
pre-commit install
```

The hooks will automatically:

- Format rust code
- Run `clippy` for linting
- Check for common issues like trailing whitespace and adding a newline at the end of files
- Prepare sqlx queries for offline mode

## Configuration

sonata uses a TOML configuration file (`sonata.toml`) for its settings. The configuration includes:

- **API settings**: Port, host, and TLS configuration for the API server
- **Gateway settings**: Port, host, and TLS configuration for the gateway server
- **Database settings**: Connection parameters for PostgreSQL
- **Logging**: Log level configuration

See `sonata.toml` for the default configuration and available options.

## License

This project is licensed under the Mozilla Public License 2.0. See the [LICENSE](LICENSE) file for details.

## Community

- **Discord**: Join our community on [Discord](https://discord.com/invite/m3FpcapGDD)
- **Website**: Visit [polyproto.org](https://polyproto.org)
- **Email**: Contact us at [info@polyphony.chat](mailto:info@polyphony.chat)
- **IRC**: See [our FAQ at the "IRC" section](https://github.com/polyphony-chat/.github/blob/main/FAQ.md#irc) for information on how to connect.


[Discord]: https://dcbadge.limes.pink/api/server/m3FpcapGDD?style=flat
<!-- [Discord]: https://img.shields.io/badge/Discord-bf63f7.svg?style=flat&logo=discord&logoColor=white-->
[Discord-invite]: https://discord.com/invite/m3FpcapGDD
[FAQ-shield]: https://img.shields.io/badge/Frequently_Asked_Questions_(FAQ)-ff62bd
[FAQ]: https://github.com/polyphony-chat/.github/blob/main/FAQ.md
