// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

// Determinism: identical inputs produce identical outputs.
r0 = __mu::math::rand(seed = 0, x = 0, y = 0, z = 0);
r1_a = __mu::math::rand(seed = 0, x = 1, y = 2, z = 3);
r1_b = __mu::math::rand(seed = 0, x = 1, y = 2, z = 3);
std::assert(r1_a == r1_b, "rand must be deterministic for identical inputs");

// Distinctness: varying seed or any coordinate produces distinct outputs.
r_seed = __mu::math::rand(seed = 1, x = 1, y = 2, z = 3);
r_x = __mu::math::rand(seed = 0, x = 2, y = 2, z = 3);
r_y = __mu::math::rand(seed = 0, x = 1, y = 3, z = 3);
r_z = __mu::math::rand(seed = 0, x = 1, y = 2, z = 4);
std::assert(r1_a != r_seed, "varying seed must change output");
std::assert(r1_a != r_x, "varying x must change output");
std::assert(r1_a != r_y, "varying y must change output");
std::assert(r1_a != r_z, "varying z must change output");

// Bounds: outputs lie in [0.0, 1.0).
r_large = __mu::math::rand(seed = 123456789, x = 987654321, y = 123456789, z = 567890123);
std::assert(r0 >= 0.0 and r0 < 1.0, "zero-input rand out of [0, 1)");
std::assert(r1_a >= 0.0 and r1_a < 1.0, "simple rand out of [0, 1)");
std::assert(r_seed >= 0.0 and r_seed < 1.0, "seeded rand out of [0, 1)");
std::assert(r_large >= 0.0 and r_large < 1.0, "large-input rand out of [0, 1)");
