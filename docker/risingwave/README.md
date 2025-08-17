```bash
# connect to risingwave
psql -h localhost -p 4566 -d dev -U root
# create database
CREATE DATABASE test;
\c test
# list all tables
\dt
# exit
\q
```