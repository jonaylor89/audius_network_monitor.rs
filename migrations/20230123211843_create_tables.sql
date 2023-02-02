-- Add migration script here
CREATE TABLE network_monitoring_index_blocks (
    run_id SERIAL NOT NULL,
    is_current BOOL NOT NULL DEFAULT FALSE,
    blocknumber INT NOT NULL,
    is_complete BOOL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (run_id)
);

CREATE TABLE network_monitoring_content_nodes (
    spID INT NOT NULL,
    endpoint VARCHAR NOT NULL DEFAULT '',
    run_id INT NOT NULL DEFAULT 0,
    CONSTRAINT fk_run_id FOREIGN KEY (run_id) REFERENCES network_monitoring_index_blocks(run_id) ON DELETE CASCADE,
    PRIMARY KEY (run_id, spID)
);

CREATE TABLE network_monitoring_users (
    user_id INT NOT NULL,
    wallet TEXT NOT NULL,
    replica_set VARCHAR NOT NULL,
    run_id INT NOT NULL,
    primary_clock_value INT NOT NULL DEFAULT 0,
    secondary1_clock_value INT NOT NULL DEFAULT 0,
    secondary2_clock_value INT NOT NULL DEFAULT 0,
    primarySpID INT NOT NULL DEFAULT 0,
    secondary1SpID INT NOT NULL DEFAULT 0,
    secondary2SpID INT NOT NULL DEFAULT 0,
    CONSTRAINT fk_run_id FOREIGN KEY (run_id) REFERENCES network_monitoring_index_blocks(run_id) ON DELETE CASCADE,
    CONSTRAINT fk_primarySpID FOREIGN KEY (run_id, primarySpID) REFERENCES network_monitoring_content_nodes(run_id, spID),
    CONSTRAINT fk_secondary1SpID FOREIGN KEY (run_id, secondary1SpID) REFERENCES network_monitoring_content_nodes(run_id, spID),
    CONSTRAINT fk_secondary2SpID FOREIGN KEY (run_id, secondary2SpID) REFERENCES network_monitoring_content_nodes(run_id, spID),
    PRIMARY KEY (run_id, user_id)
);

CREATE TABLE network_monitoring_cids_from_discovery (
    cid VARCHAR NOT NULL,
    run_id INT NOT NULL DEFAULT 0,
    user_id INT NOT NULL DEFAULT 0,
    ctype VARCHAR NOT NULL,
    CONSTRAINT fk_run_id FOREIGN KEY (run_id) REFERENCES network_monitoring_index_blocks(run_id) ON DELETE CASCADE
);

CREATE TABLE network_monitoring_cids_from_content (
    cid VARCHAR NOT NULL,
    run_id INT NOT NULL DEFAULT 0,
    user_id INT NOT NULL DEFAULT 0,
    content_node_spID INT NOT NULL DEFAULT 0,
    CONSTRAINT fk_run_id FOREIGN KEY (run_id) REFERENCES network_monitoring_index_blocks(run_id) ON DELETE CASCADE,
    CONSTRAINT fk_content_node_spID FOREIGN KEY (run_id, content_node_spID) REFERENCES network_monitoring_content_nodes(run_id, spID)
);