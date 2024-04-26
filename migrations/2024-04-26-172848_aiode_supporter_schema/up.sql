CREATE TABLE aiode_supporter (
    user_id NUMERIC(20, 0) NOT NULL UNIQUE,
    creation_timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id)
);
