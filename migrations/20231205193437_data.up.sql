INSERT INTO deciders
VALUES ('Order', 'OrderCreated'),
       ('Order', 'OrderPrepared'),
       ('Order', 'OrderNotCreated'),
       ('Order', 'OrderNotPrepared'),
       ('Restaurant', 'RestaurantCreated'),
       ('Restaurant', 'RestaurantNotCreated'),
       ('Restaurant', 'RestaurantMenuChanged'),
       ('Restaurant', 'RestaurantMenuNotChanged'),
       ('Restaurant', 'RestaurantOrderPlaced'),
       ('Restaurant', 'RestaurantOrderNotPlaced');

INSERT INTO views
VALUES ('view', 500),
       ('saga', 500);

