# gene-platform
Gene Data Analytics Platform

## Kubernetes prerequisite

- Kubernetes v1.33+
- Ingress Controller, Cert Manager, External DNS Installed

## Deployment

For local test or development:

1. Install the MySQL Client, SSL lib and pkg-config
    ``` Ubuntu
    apt install libmysqlclient-dev libssl-dev pkg-config
    ```

2. Start mysql and redis
    ```docker
    docker-compose up -d
    ```

3. Copy .env.example to .env and set it properly

4. Install `diesel cli`, download [watchexec](https://github.com/watchexec/watchexec) and init the database
    ```
    cargo install diesel_cli --no-default-features --features "mysql"
    diesel setup
    ```

5. Run the app
    ```
    watchexec -e rs -r cargo run
    yarn dev

    # or
    cargo build --release
    yarn build
    ```

For Kubernetes deployment:

1. Build and push docker images
    ```
    docker/build.sh
    ```

2. Deploy with helm chart
    ```
    cd charts
    helm upgrade gene-platform ./gene-platform -n gene-platform --install --create-namespace --wait 
    ```
