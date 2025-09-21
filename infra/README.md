# Employees Manager Deployment Guide

The infra folder contains several files to use Employees Manager application in different deployment scenarios.

The components in the application are backend, frontend and database and can be used as independent docker containers, external services or embedded in the same container.

## Three components docker compose

In this setting, each component is a separate docker container that exposes a port and communicated with others.
It is good for local testing but is not recommended for cloud or production environments because the database does not have replicas or backups.

In order to use it, you need to build the frontend and backend docker images using `Dockerfile-frontend` and `Dockerfile-backend` with the following commands:

Frontend

```sh
make build-local-docker-fe
```

Backend:

```sh
make build-local-docker-be
```

Then, you can run the docker compose with the file `docker-compose-external.yml` with the command

```sh
docker compose -f docker-compose-external.yml up -d
```

After everything is up, you can configure the database with

```sh
make setup-mongo-local-docker-compose
```

Now you can access the web app to `localhost:4200`

## Two components docker compose

In this setting, frontend is integrated in the backend that serves web app pages as static files.
It is good for local testing but is not recommended for cloud or production environments because the database does not have replicas or backups.

You only need to build image for the application with the command

```sh
make build-local-integrated-docker
```

Then, you can run the docker compose with the file `docker-compose-integrated.yml` with the command

```sh
docker compose -f docker-compose-integrated.yml up -d
```

After everything is up, you can configure the database with

```sh
make setup-mongo-local-docker-compose
```

Now you can access the web app to `localhost:3000`
