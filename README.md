## Usage

1. Build

    ```shell
    cargo build --release
    ```

2. Set up your .env

    ```shell
    cp .env.sample .env
    ```

   Fill in the urls/tokens

    ```
    YELLOWSTONE_GRPC_URL=
    YELLOWSTONE_GRPC_TOKEN=
    
    LASERSTREAM_URL=
    LASERSTREAM_TOKEN=
    ```

3. Run

    ```shell
    ./target/release/laserbench simulate --path grpc.csv --grpc
    ```

    ```shell
    ./target/release/laserbench simulate --path laser.csv --laser
    ```

