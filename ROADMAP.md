# 🗺️ Roadmap: AI-Assisted & Organic Procedural Modeling for µcad

This fork will extend **µcad** from a code-first parametric CAD language into an AI-assisted, local-first 3D modeling ecosystem. My goal is to bridge declarative CAD precision with natural-language generation, organic procedural geometry, and seamless downstream game/3D engine workflows.

---

## Q1: Procedural Primitives & Benchmark Suite
*Build the geometric foundation and establish strict quality metrics.*
- [ ] **Organic & Parametric Primitive Library:** Finalize and open-source reusable µcad primitives for branching structures, surface sweeps, and connected foliage.
- [ ] **Standardized Evaluation Suite:** Publish a benchmark dataset spanning standard mechanical CAD parts and complex organic stress tests (e.g., mangrove root/branch systems).
- [ ] **Automated Validation Metrics:** Benchmark geometric continuity and compile validity across the test suite.

## Q2: Constrained LLM Generation & Compiler Feedback Loop
*Make natural-language-to-µcad generation deterministic and type-safe.*
- [ ] **Rust Orchestration Pipeline:** Pair open-weights LLMs with grammar-constrained decoding to guarantee syntactically valid µcad output.
- [ ] **Compiler-Error Feedback Loop:** Implement automated error parsing so models self-correct type and geometry errors iteratively.
- [ ] **Local Model Benchmarks:** Measure and publish first-pass vs. self-corrected compilation rates across local open-weights models.

## Q3: No-Code Interactive Modeling Prototype
*Lower the barrier to entry with visual, real-time parametric control.*
- [ ] **Local-First Modeling UI:** Release an interactive prototype for generating 3D models directly from natural-language prompts.
- [ ] **Visual Parametric Controls:** Expose generated dimensional parameters as interactive sliders/controls so creators can tweak geometry without writing code.
- [ ] **Parametric History Inspector:** Allow users to inspect, step through, and modify the underlying modeling tree visually.

## Q4: Ecosystem Integrations & Community Release
*Connect µcad assets to standard 3D pipelines and empower the community.*
- [ ] **3D Format & Engine Exporters:** Publish open-source export workflows for **glTF**, **OpenUSD**, **Blender**, and **Bevy**.
- [ ] **Learning Resources:** Release comprehensive documentation and curated example libraries covering both mechanical and organic workflows.
- [ ] **Community Workshops:** Host onboarding sessions and guides tailored for makers, educators, and Rust 3D developers.

---

## 🤝 Contributing & Feedback
This roadmap reflects our current high-level vision and may evolve as milestones are tested. Have ideas, model benchmarks, or integration requests? Feel free to open an Issue or start a Discussion!
