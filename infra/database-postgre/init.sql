-- Create the database emdb
CREATE DATABASE emdb;
-- connect to the emdb database
\c emdb;

-- Create the table for users
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(100) NOT NULL,
    surname VARCHAR(100) NOT NULL,
    api_key VARCHAR(255),
    platform_admin BOOLEAN NOT NULL
);

-- Create the table for companies
CREATE TABLE companies (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE
);

-- Create the table for user company assignments
CREATE TABLE user_company_assignments (
    id SERIAL PRIMARY KEY,
    app_user INT NOT NULL,
    company INT NOT NULL,
    role VARCHAR(50) NOT NULL,
    job_title VARCHAR(100) NOT NULL,
    FOREIGN KEY (app_user) REFERENCES users(id),
    FOREIGN KEY (company) REFERENCES companies(id)
);

-- Create the table for company management team
CREATE TABLE company_management_teams (
    id SERIAL PRIMARY KEY,
    company INT NOT NULL,
    app_user INT NOT NULL,
    FOREIGN KEY (company) REFERENCES companies(id),
    FOREIGN KEY (app_user) REFERENCES users(id)
);

-- Create the database user
CREATE USER db_user WITH PASSWORD 'my_password';

-- Grand privileges
GRANT CONNECT ON DATABASE emdb TO db_user;
GRANT USAGE ON SCHEMA public TO db_user;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO db_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO db_user;

-- Insert administrator user inside users table
INSERT INTO users (email, username, password_hash, name, surname, api_key, platform_admin)
VALUES (
    'admin@app.com',
    'admin',
    '$2a$12$cyWTPScPg8Z9CEkiq7tz8.ruaPcELfZZ/mCJZke9Xgy43tqhhhWMO', -- bycript for "password"
    'Admin',
    'User',
    NULL,
    TRUE
);