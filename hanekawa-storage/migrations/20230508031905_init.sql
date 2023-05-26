CREATE TABLE peer_announces(
       info_hash bytea NOT NULL,
       peer_id bytea NOT NULL,
       ip inet NOT NULL,
       port integer NOT NULL,
       uploaded bigint NOT NULL,
       downloaded bigint NOT NULL,
       remaining bigint NOT NULL,
       event text,
       last_update_ts timestamptz,
       PRIMARY KEY(info_hash, peer_id)
);
