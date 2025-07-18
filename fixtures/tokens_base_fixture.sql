-- Base fixture for token testing scenarios
-- Contains common database setup that can be reused across multiple test scenarios

-- Algorithm identifiers for public keys
INSERT INTO algorithm_identifiers (id, algorithm_identifier, common_name, parameters) VALUES
(1, 'rsaEncryption', 'RSA', NULL),
(2, 'id-ecPublicKey', 'EC', NULL);

-- Test actors (users 1-4 to support various test scenarios)
-- First insert into actors table with type 'local'
INSERT INTO actors (uaid, type) VALUES
('00000000-0000-0000-0000-000000000001', 'local'),
('00000000-0000-0000-0000-000000000002', 'local'),
('00000000-0000-0000-0000-000000000003', 'local'),
('00000000-0000-0000-0000-000000000004', 'local');

-- Then insert into local_actors with the specific local actor data
INSERT INTO local_actors (uaid, local_name, deactivated, joined, password_hash) VALUES
('00000000-0000-0000-0000-000000000001', 'test_user_1', FALSE, NOW(), 'hash'),
('00000000-0000-0000-0000-000000000002', 'test_user_2', FALSE, NOW(), 'hash'),
('00000000-0000-0000-0000-000000000003', 'test_user_3', FALSE, NOW(), 'hash'),
('00000000-0000-0000-0000-000000000004', 'test_user_4', FALSE, NOW(), 'hash');

-- Test public keys (corresponding to all test users)
INSERT INTO public_keys (id, uaid, pubkey, algorithm_identifier) VALUES
(1, '00000000-0000-0000-0000-000000000001', 'test_pubkey_1', 1),
(2, '00000000-0000-0000-0000-000000000002', 'test_pubkey_2', 1),
(3, '00000000-0000-0000-0000-000000000003', 'test_pubkey_3', 1),
(4, '00000000-0000-0000-0000-000000000004', 'test_pubkey_4', 1),
-- Additional public keys for multiple tokens per user testing
(5, '00000000-0000-0000-0000-000000000001', 'test_pubkey_1_b', 1),
(6, '00000000-0000-0000-0000-000000000004', 'test_pubkey_4_b', 1);

-- Test ID-CSRs (for all test users)
INSERT INTO idcsr (
    id, serial_number, uaid, actor_public_key_id, actor_signature,
    session_id, valid_not_before, valid_not_after, extensions, pem_encoded
) VALUES
(1, 12345678901234567890, '00000000-0000-0000-0000-000000000001', 1, 'test_signature_1', 'test_session_1', NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 'test_extensions_1', 'test_csr_pem_1'),
(2, 98765432109876543210, '00000000-0000-0000-0000-000000000002', 2, 'test_signature_2', 'test_session_2', NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 'test_extensions_2', 'test_csr_pem_2'),
(3, 11111111111111111111, '00000000-0000-0000-0000-000000000003', 3, 'test_signature_3', 'test_session_3', NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 'test_extensions_3', 'test_csr_pem_3'),
(4, 55555555555555555555, '00000000-0000-0000-0000-000000000004', 4, 'test_signature_4', 'test_session_4', NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 'test_extensions_4', 'test_csr_pem_4'),
-- Additional ID-CSR with unique serial number for user 1 (for testing multiple tokens same user)
(5, 12345678901234567891, '00000000-0000-0000-0000-000000000001', 5, 'test_signature_1_b', 'test_session_1_b', NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 'test_extensions_1_b', 'test_csr_pem_1_b'),
-- Additional ID-CSR with unique serial number for user 4 (for testing expired tokens)
(6, 55555555555555555556, '00000000-0000-0000-0000-000000000004', 6, 'test_signature_4_b', 'test_session_4_b', NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 'test_extensions_4_b', 'test_csr_pem_4_b');

-- Test issuers (must be inserted after idcsr because of foreign key constraint)
INSERT INTO issuers (id, domain_components, pem_encoded) VALUES
(1, ARRAY['example', 'com'], 'test_csr_pem_1'),
(2, ARRAY['test', 'org'], 'test_csr_pem_2');

-- Test ID-Certs
-- User 1 and 2 have certificates (for basic scenarios)
-- User 4 has a certificate (for extended scenarios)
-- User 3 has no certificate (for testing scenarios where users lack certificates)
INSERT INTO idcert (
    idcsr_id, issuer_info_id, valid_not_before, valid_not_after,
    home_server_public_key_id, home_server_signature, pem_encoded
) VALUES
(1, 1, NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 1, 'test_home_server_sig_1', 'test_cert_pem_1'),
(2, 2, NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 2, 'test_home_server_sig_2', 'test_cert_pem_2'),
(4, 1, NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 4, 'test_home_server_sig_4', 'test_cert_pem_4'),
-- Additional certificate for testing multiple tokens same user (corresponding to additional ID-CSR)
(5, 1, NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 5, 'test_home_server_sig_1_b', 'test_cert_pem_1_b'),
-- Additional certificate for user 4 (corresponding to additional ID-CSR for expired tokens)
(6, 1, NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 6, 'test_home_server_sig_4_b', 'test_cert_pem_4_b');
