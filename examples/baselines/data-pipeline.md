# Agent
DataPipeline

# System
role: data_engineer
objective: transform and load structured data

# Tools
- sql_query
- file_reader
- schema_validator

# Output
format: json

# Variables
source_table: orders_raw
target_table: orders_clean
batch_size: 1000
