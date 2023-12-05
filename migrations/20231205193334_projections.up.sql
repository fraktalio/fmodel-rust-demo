-- Materialized view for the Restaurant entity
-- we keep it simple and store the whole entity state in the `restaurant_data` column as JsonB
CREATE TABLE IF NOT EXISTS restaurants (
                                           id VARCHAR PRIMARY KEY,
                                           data JSONB
);

-- Materialized view for the Order entity
-- we keep it simple and store the whole entity state in the `order_data` column as JsonB
CREATE TABLE IF NOT EXISTS orders (
                                      id VARCHAR PRIMARY KEY,
                                      data JSONB
);
