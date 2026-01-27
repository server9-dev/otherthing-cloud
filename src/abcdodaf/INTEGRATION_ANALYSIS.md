# egui-snarl Integration Analysis

## Overview
Analysis of whether to vendor egui-snarl directly into abcdodaf or continue using it as a dependency.

## Source Code Analysis

### Repository
- **Source**: https://github.com/zakarumych/egui-snarl
- **License**: MIT OR Apache-2.0 (very permissive)
- **Version**: 0.9.0
- **Lines of Code**: ~2,900 lines total

### File Structure
```
src/
├── lib.rs           (~700 lines) - Core Snarl data structure
├── ui.rs            (~2,200 lines) - Main UI rendering logic
└── ui/
    ├── wire.rs      - Wire rendering
    ├── viewer.rs    - SnarlViewer trait
    ├── state.rs     - UI state management
    ├── scale.rs     - Scaling utilities
    ├── pin.rs       - Pin management
    ├── effect.rs    - Visual effects
    └── background_pattern.rs - Background rendering
```

### Dependencies
- egui (0.33) - core UI framework
- egui-scale (0.3.0) - scaling utilities
- slab (0.4) - efficient ID allocation
- smallvec (1.15) - stack-allocated vectors
- serde (optional) - serialization

## Integration Approaches

### Option 1: Continue as Dependency (Current)
**Pros:**
- ✅ Automatic upstream bug fixes and improvements
- ✅ Minimal maintenance overhead
- ✅ Clear separation of concerns
- ✅ Smaller codebase to maintain
- ✅ Community support and documentation

**Cons:**
- ❌ Limited ability to modify core behavior
- ❌ Breaking changes in updates require adaptation
- ❌ Can't customize internal rendering logic
- ❌ Dependency on external maintainer

**Best for:** Standard node-graph use cases where egui-snarl's API suffices

### Option 2: Vendor with Attribution
**Pros:**
- ✅ Full control over implementation
- ✅ Can modify core behavior for BPMN/DoDAF compliance
- ✅ No dependency on external updates
- ✅ Tight integration with domain types
- ✅ Can optimize specifically for our use case
- ✅ License allows this (MIT/Apache-2.0)

**Cons:**
- ❌ Must maintain ~2,900 lines of additional code
- ❌ Miss upstream improvements unless manually merged
- ❌ Responsibility for bug fixes
- ❌ Larger codebase
- ❌ May drift from upstream over time

**Best for:** When core modifications are needed for BPMN/DoDAF standards

### Option 3: Hybrid Approach (Recommended)
**Pros:**
- ✅ Use egui-snarl as dependency initially
- ✅ Build custom wrapper types for BPMN/DoDAF
- ✅ Only vendor if we hit limitations
- ✅ Test customization needs before committing
- ✅ Can transition to vendoring later if needed

**Implementation:**
1. Keep `egui-snarl = "0.9"` as dependency
2. Create BPMN-specific types that implement `SnarlViewer`
3. Add DoDAF metadata as wrapper structs
4. Extend functionality through composition, not modification
5. Document any limitations encountered
6. Vendor only if we need to modify core rendering/behavior

## Recommendation: Hybrid Approach

### Rationale
1. **Current needs are met**: Our BpmnNode and BpmnViewer work well with egui-snarl's API
2. **BPMN/DoDAF compliance**: Can be achieved through domain types, not library modification
3. **Risk mitigation**: If we hit limitations, vendoring is still an option
4. **Maintainability**: Keeping dependency reduces maintenance burden
5. **Upstream benefits**: Get bug fixes and improvements for free

### When to Vendor (Future Criteria)
Vendor egui-snarl if we need to:
- Modify wire rendering to match BPMN arrow specifications
- Change node layout beyond what SnarlViewer provides
- Add BPMN-specific gestures or interactions
- Integrate DoDAF metadata at the rendering level
- Optimize performance for large process models

### Attribution if Vendoring
If we decide to vendor later, include:
```rust
// Originally from egui-snarl v0.9.0
// Copyright (c) 2023 Zakarum
// Licensed under MIT OR Apache-2.0
// Source: https://github.com/zakarumych/egui-snarl
// Modified for BPMN 2.0 and DoDAF 2.02 compliance
```

## Next Steps
1. ✅ Keep egui-snarl as dependency
2. Enhance BpmnNode types to fully match BPMN 2.0 spec
3. Add DoDAF operational activity metadata
4. Implement BPMN export/import (XML)
5. Create comprehensive test suite
6. Document any egui-snarl limitations
7. Revisit vendoring decision if core modifications become necessary

## References
- [egui-snarl GitHub](https://github.com/zakarumych/egui-snarl)
- [BPMN 2.0 Specification](https://www.omg.org/spec/BPMN/2.0/)
- [DoDAF 2.02](https://dodcio.defense.gov/Library/DoD-Architecture-Framework/)
