-- Specific fixture for token validation testing scenarios
-- This builds on tokens_base_fixture.sql and only adds the token-specific data

-- Test user tokens focused on expiration and validity scenarios
INSERT INTO user_tokens (token_hash, cert_id, uaid, valid_not_after) VALUES
-- User 1: mix of expired and valid tokens for testing expiration logic
('expired_token_hash_1', 1, '00000000-0000-0000-0000-000000000001', NOW() - INTERVAL '1 hour'),
('valid_token_hash_1', 1, '00000000-0000-0000-0000-000000000001', NOW() + INTERVAL '1 hour'),

-- User 2: multiple valid tokens with different expiration times (for testing "latest" selection)
('valid_token_hash_2', 2, '00000000-0000-0000-0000-000000000002', NOW() + INTERVAL '2 hours'),
('valid_token_hash_3', 2, '00000000-0000-0000-0000-000000000002', NOW() + INTERVAL '30 minutes');
