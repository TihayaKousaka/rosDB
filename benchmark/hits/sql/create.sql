DROP TABLE IF EXISTS hits;
CREATE EXTERNAL TABLE hits
STORED AS PARQUET
LOCATION '/home/cnosdb_dev/hits.parquet';
