-- Specific fixture for token serial lookup testing scenarios
-- This builds on tokens_base_fixture.sql and only adds the token-specific data

-- Test user tokens focused on serial lookup scenarios
INSERT INTO user_tokens (token_hash, cert_id, uaid, valid_not_after) VALUES
-- User 1: multiple tokens (testing multiple tokens returning same serial)
('token_hash_user_1_a', 1, '00000000-0000-0000-0000-000000000001', NOW() + INTERVAL '1 hour'),
('token_hash_user_1_b', 5, '00000000-0000-0000-0000-000000000001', NOW() + INTERVAL '2 hours'),

-- User 2: multiple tokens (testing cross-user serial differentiation)
('token_hash_user_2_a', 2, '00000000-0000-0000-0000-000000000002', NOW() + INTERVAL '1 hour'),

-- User 4: tokens including expired ones (testing that serial lookup works regardless of expiration)
('token_hash_user_4_a', 4, '00000000-0000-0000-0000-000000000004', NOW() + INTERVAL '1 hour'),
-- Insert expired token as non-expired first to avoid auto-cleanup trigger
('expired_token_hash_user_4', 6, '00000000-0000-0000-0000-000000000004', NOW() + INTERVAL '1 hour');

-- Update token to be expired after insertion (workaround for cleanup trigger)
UPDATE user_tokens
SET valid_not_after = NOW() - INTERVAL '1 hour'
WHERE token_hash = 'expired_token_hash_user_4';
