-- sql/create_service_account.sql


CREATE ROLE elmo_app WITH LOGIN PASSWORD 'password';

GRANT CONNECT ON DATABASE elmo TO elmo_app;

\c elmo

GRANT USAGE ON SCHEMA oscar TO elmo_app;

GRANT SELECT ON ALL TABLES IN SCHEMA oscar TO elmo_app;

ALTER DEFAULT PRIVILEGES IN SCHEMA oscar GRANT SELECT ON TABLES TO elmo_app;

