# Doclytics

## Description

This is a Rust project that interacts with the paperless-ngx API to fetch and update documents. The goal is to utilize local llms, in this case ollama, to generate metadata for the documents in your paperless document library.
It uses the `reqwest` library to make HTTP requests and `serde_json` for JSON serialization and deserialization, as well as ollama_rs

## Setup

1. Install Rust: Follow the instructions on the [official Rust website](https://www.rust-lang.org/tools/install) to install Rust on your machine.

2. Clone the repository: `git clone https://github.com/yourusername/yourrepository.git`

3. Navigate to the project directory: `cd doclytics`

4. Build the project: `cargo build`

5. Run the project: `cargo run`

## Environment Variables

This project uses the following environment variables:

- `TOKEN`: This is used for API authentication.

Set these in a `.env` file in the root of your project.

## Usage

This project is currently set up to fetch and update documents from an API. The main function queries custom fields from the API, generates a request to the Ollama service, and updates the document fields based on the response.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License

[MIT](https://choosealicense.com/licenses/mit/)
