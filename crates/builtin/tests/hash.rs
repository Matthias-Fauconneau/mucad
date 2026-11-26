use mucad_builtin::mu::*;
use mucad_macros::test_builtin_fn;

// --- Determinism Tests ---

// rand(seed=0, x=0, y=0, z=0) should give deterministic result
#[test]
#[test_builtin_fn(math::rand(seed = 0, x = 0, y = 0, z = 0) == 0.0)]
fn rand_deterministic_zero() {
    // Same inputs always produce same output
}

// rand(seed=0, x=1, y=2, z=3) deterministic
#[test]
#[test_builtin_fn(math::rand(seed = 0, x = 1, y = 2, z = 3) == 0.64593310462126085625)]
fn rand_deterministic_simple() {
    // Simple deterministic test
}

// Different seeds produce different results
#[test]
#[test_builtin_fn(math::rand(seed = 1, x = 0, y = 0, z = 0) == 0.4071559758835981979)]
fn rand_different_seed() {
    // Different seed should produce different output
}

// Test with varied inputs
#[test]
#[test_builtin_fn(math::rand(seed = 42, x = 10, y = 20, z = 30) == 0.4829727331561427528)]
fn rand_with_seed42() {
    // Test with seed 42
}

// Edge case: minimal seed
#[test]
#[test_builtin_fn(math::rand(seed = 1, x = 1, y = 1, z = 1) == 0.35885157650225574244)]
fn rand_minimal() {
    // Minimal positive values
}

// Large values test
#[test]
#[test_builtin_fn(math::rand(seed = 123456789, x = 987654321, y = 123456789, z = 567890123) == 0.72570234435964953157)]
fn rand_large_values() {
    // Large values to test mixing
}