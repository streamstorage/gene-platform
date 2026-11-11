# gene-platform
Gene Data Analytics Platform

## Kubernetes prerequisite

- Kubernetes v1.33+
- Ingress Controller, Cert Manager, External DNS Installed

## Deployment

Local deployment:

1. Install Postgres lib, SSL lib and pkg-config
    ```
    # For Ubuntu
    apt install libpq-dev libssl-dev pkg-config
    ```

2. Run Postgre and redis in docker
    ```
    cd docker
    docker-compose up -d
    ```

3. Copy .env.example to .env and set it properly

4. Install `diesel cli`, then init the database
    ```
    cargo install diesel_cli --no-default-features --features "postgres"
    diesel setup
    ```

5. Download [watchexec](https://github.com/watchexec/watchexec), and then debug or build the app
    ```
    watchexec -e rs -r cargo run
    yarn dev

    # or
    cargo build --release
    yarn build
    ```

Kubernetes deployment:

1. Build and push docker images
    ```
    docker/build.sh
    ```

2. Deploy with helm chart
    ```
    cd charts
    helm upgrade gene-platform ./gene-platform -n gene-platform --install --create-namespace --wait 
    ```
