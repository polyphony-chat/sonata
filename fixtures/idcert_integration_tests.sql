-- Fixture for IdCert integration tests
-- Contains data for testing HomeServerCert::get_idcert_by method

-- Algorithm identifiers including ED25519
INSERT INTO algorithm_identifiers (id, algorithm_identifier, common_name, parameters_der_encoded) VALUES
(1, 'rsaEncryption', 'RSA', NULL),
(2, 'id-ecPublicKey', 'EC', NULL),
(3, '1.3.101.112', 'Edwards-curve Digital Signature Algorithm (EdDSA) Ed25519', NULL);

-- Test actors for IdCert scenarios
INSERT INTO actors (uaid, type) VALUES
('00000000-0000-0000-0000-000000000010', 'local'),
('00000000-0000-0000-0000-000000000011', 'local'),
('00000000-0000-0000-0000-000000000012', 'local'),
('00000000-0000-0000-0000-000000000013', 'local');

INSERT INTO local_actors (uaid, local_name, deactivated, joined, password_hash) VALUES
('00000000-0000-0000-0000-000000000010', 'idcert_test_user_1', FALSE, NOW(), 'hash'),
('00000000-0000-0000-0000-000000000011', 'idcert_test_user_2', FALSE, NOW(), 'hash'),
('00000000-0000-0000-0000-000000000012', 'idcert_test_user_3', FALSE, NOW(), 'hash'),
('00000000-0000-0000-0000-000000000013', 'idcert_test_user_4', FALSE, NOW(), 'hash');

-- Test public keys (using placeholders - tests will generate real keys)
INSERT INTO public_keys (id, uaid, pubkey, algorithm_identifier) VALUES
(100, '00000000-0000-0000-0000-000000000010', 'PLACEHOLDER_PEM_KEY_1', 3),
(101, '00000000-0000-0000-0000-000000000011', 'PLACEHOLDER_PEM_KEY_2', 3),
(102, '00000000-0000-0000-0000-000000000012', 'PLACEHOLDER_PEM_KEY_3', 3),
(103, '00000000-0000-0000-0000-000000000013', 'PLACEHOLDER_PEM_KEY_4', 3),
-- Additional home server public keys
(200, NULL, 'PLACEHOLDER_HOMESERVER_KEY_1', 3),
(201, NULL, 'PLACEHOLDER_HOMESERVER_KEY_2', 3);

-- Test ID-CSRs for IdCert tests
INSERT INTO idcsr (
    id, serial_number, uaid, subject_public_key_id, subject_signature,
    session_id, valid_not_before, valid_not_after, extensions, pem_encoded
) VALUES
(100, 10000000000000000001, '00000000-0000-0000-0000-000000000010', 100, 'test_signature_idcert_1', 'session_idcert_1', NOW() - INTERVAL '1 day', NOW() + INTERVAL '30 days', 'test_extensions_idcert_1', 'test_csr_pem_idcert_1'),
(101, 10000000000000000002, '00000000-0000-0000-0000-000000000011', 101, 'test_signature_idcert_2', 'session_idcert_2', NOW() - INTERVAL '1 day', NOW() + INTERVAL '30 days', 'test_extensions_idcert_2', 'test_csr_pem_idcert_2'),
(102, 10000000000000000003, '00000000-0000-0000-0000-000000000012', 102, 'test_signature_idcert_3', 'session_idcert_3', NOW() - INTERVAL '2 days', NOW() - INTERVAL '1 day', 'test_extensions_idcert_3', 'test_csr_pem_idcert_3'),
(103, 10000000000000000004, '00000000-0000-0000-0000-000000000013', 103, 'test_signature_idcert_4', 'session_idcert_4', NOW() + INTERVAL '1 day', NOW() + INTERVAL '30 days', 'test_extensions_idcert_4', 'test_csr_pem_idcert_4');

-- Test issuers (different domains for testing)
INSERT INTO issuers (id, domain_components) VALUES
(100, ARRAY['example', 'com']),
(101, ARRAY['test', 'org']),
(102, ARRAY['expired', 'net']);

-- Test ID-Certs
-- Valid certificate for example.com
INSERT INTO idcert (
    idcsr_id, issuer_info_id, valid_not_before, valid_not_after,
    home_server_public_key_id, home_server_signature, pem_encoded
) VALUES
(100, 100, NOW() - INTERVAL '1 day', NOW() + INTERVAL '30 days', 200, 'homeserver_signature_1', 'PLACEHOLDER_CERT_PEM_1'),
-- Valid certificate for test.org
(101, 101, NOW() - INTERVAL '1 day', NOW() + INTERVAL '30 days', 201, 'homeserver_signature_2', 'PLACEHOLDER_CERT_PEM_2'),
-- Expired certificate for expired.net
(102, 102, NOW() - INTERVAL '2 days', NOW() - INTERVAL '1 day', 200, 'homeserver_signature_3', 'PLACEHOLDER_CERT_PEM_3');
