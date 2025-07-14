-- Fixture for LocalActor testing scenarios
-- This builds on the basic actor setup and adds specific test data for LocalActor methods

-- Algorithm identifiers for public keys (needed for actor setup)
INSERT INTO algorithm_identifiers (id, algorithm_identifier, common_name, parameters) VALUES
(1, 'rsaEncryption', 'RSA', NULL),
(2, 'id-ecPublicKey', 'EC', NULL);

-- Test actors for various scenarios
-- First insert into actors table with type 'local'
INSERT INTO actors (uaid, type) VALUES
('00000000-0000-0000-0000-000000000001', 'local'),
('00000000-0000-0000-0000-000000000002', 'local'),
('00000000-0000-0000-0000-000000000003', 'local'),
('00000000-0000-0000-0000-000000000004', 'local'),
('00000000-0000-0000-0000-000000000005', 'local');

-- Then insert into local_actors with specific test data
INSERT INTO local_actors (uaid, local_name, deactivated, joined, password_hash) VALUES
-- Active users
('00000000-0000-0000-0000-000000000001', 'alice', FALSE, '2023-01-01 12:00:00', 'hash'),
('00000000-0000-0000-0000-000000000002', 'bob', FALSE, '2023-01-02 12:00:00', 'hash'),
('00000000-0000-0000-0000-000000000003', 'charlie', FALSE, '2023-01-03 12:00:00', 'hash'),
-- Deactivated user
('00000000-0000-0000-0000-000000000004', 'deactivated_user', TRUE, '2023-01-04 12:00:00', 'hash'),
-- User with special characters in name
('00000000-0000-0000-0000-000000000005', 'user_with_underscores', FALSE, '2023-01-05 12:00:00', 'hash');
