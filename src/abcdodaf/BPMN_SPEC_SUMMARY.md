# BPMN 2.0 Specification Summary for Visual Editor

## Official Specification
- **Source**: [OMG BPMN 2.0 Specification](http://www.omg.org/spec/BPMN/2.0/)
- **Version**: 2.0.2 (January 2014)
- **Standard**: ISO/IEC 19510
- **Name**: Business Process Model **and** Notation (changed from "Modeling")

## 1. Node Types

### 1.1 Events (Circles)

**Visual Indicators:**
- Start Events: Single thin circle
- Intermediate Events: Double thin circles
- End Events: Single thick circle
- Non-interrupting: Dashed border
- Interrupting: Solid border

**Event Types (12 total):**
1. **None** - Basic start/end
2. **Message** - Receives/sends messages
3. **Timer** - Time-based triggers (clock icon)
4. **Signal** - Broadcast (triangle icon)
5. **Error** - Business exceptions (lightning bolt)
6. **Escalation** - Reports to parent
7. **Cancel** - Transaction cancellation
8. **Compensation** - Reverses activities
9. **Conditional** - Condition-based
10. **Link** - Off-page connectors
11. **Terminate** - Ends all activities
12. **Multiple** - Multiple triggers

### 1.2 Activities/Tasks (Rounded Rectangles)

**Task Types (8 types):**
1. **Abstract (None)** - Generic task
2. **User Task** - Human performer (person icon)
3. **Service Task** - Automated service (gear icon)
4. **Manual Task** - Human work without system (hand icon)
5. **Script Task** - Script execution (scroll icon)
6. **Business Rule Task** - Rules engine (table icon)
7. **Send Task** - Sends message (filled envelope)
8. **Receive Task** - Receives message (empty envelope)

**Task Markers (bottom center):**
- Loop - Sequential repetition (circular arrow)
- Multi-Instance - Parallel instances (three lines)
- Compensation - Compensation handler

### 1.3 Subprocesses

**Types (5 types):**
1. **Embedded Subprocess** - Contained within parent
2. **Call Activity** - Reusable (bold border)
3. **Event Subprocess** - Event-triggered (dashed border)
4. **Transaction Subprocess** - All-or-nothing (double border)
5. **Ad-Hoc Subprocess** - Flexible execution (tilde ~)

### 1.4 Gateways (Diamonds)

**Types (6 types):**
1. **Exclusive (XOR)** - One path (X or empty)
2. **Parallel (AND)** - All paths (+ marker)
3. **Inclusive (OR)** - One or more paths (O marker)
4. **Event-Based** - Events decide (pentagon icon)
5. **Parallel Event-Based** - Multiple events
6. **Complex** - Custom logic (* marker)

## 2. Connection Types

### 2.1 Sequence Flow
- **Visual**: Solid line with filled arrowhead
- **Purpose**: Execution order within single pool
- **Rules**: Cannot cross pool boundaries

**Key Attributes:**
- `id` (required)
- `sourceRef` (required)
- `targetRef` (required)
- `name` (optional)
- `conditionExpression` (optional)

### 2.2 Message Flow
- **Visual**: Dashed line with open arrowhead and circle at start
- **Purpose**: Message exchange between pools
- **Rules**: Only connects different pools

**Key Attributes:**
- `id` (required)
- `sourceRef` (required)
- `targetRef` (required)
- `messageRef` (optional)

### 2.3 Association
- **Visual**: Dotted line (no filled arrow)
- **Purpose**: Links artifacts to flow objects
- **Types**: None, One, Both (direction)
- **Rules**: Documentation only, no flow impact

### 2.4 Data Association
- **Visual**: Dashed line with open arrowhead
- **Purpose**: Data flow between activities and data objects/stores
- **Rules**: Does not follow sequence flow logic

## 3. Data Objects and Artifacts

### 3.1 Data Objects
- **Visual**: Rectangle with folded corner
- **Types**: Single data item, Collection (stacked)
- **States**: Can have states (e.g., "Draft", "Approved")

### 3.2 Data Store
- **Visual**: Cylinder (database icon)
- **Purpose**: Persistent storage beyond process scope

### 3.3 Data Input/Output
- **Purpose**: Define process/activity interfaces (parameters)
- **Types**: Input, Output, Collections

### 3.4 Text Annotation
- **Visual**: Open rectangle (left side open)
- **Purpose**: Add comments and notes
- **Connection**: Use Association

### 3.5 Group
- **Visual**: Dashed rounded rectangle
- **Purpose**: Visual grouping (no flow impact)
- **Rules**: Can span multiple pools

## 4. Pools and Lanes

### 4.1 Pools
- **Visual**: Large rectangle container
- **Purpose**: Represents participant (organization, role, system)
- **Types**: Black Box (collapsed), White Box (expanded)
- **Rules**: Sequence flows cannot cross boundaries

### 4.2 Lanes
- **Visual**: Subdivisions within pool
- **Purpose**: Organize by role/function/responsibility
- **Rules**: Can nest, stretch full pool width/height

## 5. Required Attributes

### Core Attributes (All Elements)
**Required:**
- `id` - Unique identifier

**Optional:**
- `name` - Human-readable name
- `documentation` - Description text

### Activities
**Required:**
- `id`

**Optional:**
- `name`, `default`, `ioSpecification`, `properties`
- `dataInputAssociations`, `dataOutputAssociations`
- `resources`, `loopCharacteristics`, `isForCompensation`

### Events
**Required:**
- `id`

**Optional:**
- `name`, `eventDefinitions`, `dataOutput`, `dataOutputAssociation`

### Gateways
**Required:**
- `id`

**Optional:**
- `name`, `gatewayDirection`, `default`

### Connections
**Sequence Flow:**
- Required: `id`, `sourceRef`, `targetRef`
- Optional: `name`, `conditionExpression`, `isImmediate`

**Message Flow:**
- Required: `id`, `sourceRef`, `targetRef`
- Optional: `name`, `messageRef`

## 6. BPMN Diagram Interchange (DI)

### Key DI Elements

**BPMNShape (for nodes):**
```xml
<bpmndi:BPMNShape id="shape_task1" bpmnElement="task1">
  <dc:Bounds x="100" y="80" width="100" height="80"/>
</bpmndi:BPMNShape>
```

**Required:**
- `id`, `bpmnElement`, `bounds` (x, y, width, height)
- **Only positive coordinates allowed**

**BPMNEdge (for connections):**
```xml
<bpmndi:BPMNEdge id="edge_flow1" bpmnElement="flow1">
  <di:waypoint x="200" y="120"/>
  <di:waypoint x="250" y="120"/>
</bpmndi:BPMNEdge>
```

**Required:**
- `id`, `bpmnElement`, `waypoint` list (minimum 2 points)

### Standard Dimensions
- **Tasks**: 100x80 units, corner radius 10
- **Events**: 36 units diameter
- **Gateways**: 50x50 units
- **Pools**: Label width 30 units, min height 150
- **Lanes**: Label width 30 units, min height 100

## 7. Implementation Checklist

### Canvas Management
- ✅ Pan and zoom
- ✅ Grid and snapping
- ✅ Positive-only coordinate system
- ✅ Multi-level undo/redo

### Element Creation
- ✅ Palette with all element types
- ✅ Drag-and-drop creation
- ✅ Automatic ID generation
- ✅ Context menus

### Visual Editing
- ✅ Move, resize, rotate
- ✅ Waypoint editing
- ✅ Alignment tools
- ✅ Z-order management

### Property Panel
- ✅ ID, name, documentation
- ✅ Type-specific properties
- ✅ Condition expressions
- ✅ Data associations
- ✅ Loop/multi-instance config

### Validation
- ✅ Semantic rules (flows within pools)
- ✅ Required attributes
- ✅ Positive coordinates
- ✅ Connection endpoints
- ✅ Gateway logic

### Import/Export
- ✅ Read/Write BPMN 2.0 XML
- ✅ Preserve DI information
- ✅ Handle unknown extensions

### Visual Indicators
- ✅ Task type icons (top-left)
- ✅ Task markers (bottom-center)
- ✅ Event type icons (inside circle)
- ✅ Gateway markers (inside diamond)
- ✅ Subprocess markers
- ✅ Boundary events
- ✅ Data object states
- ✅ Collection indicators

## References

### Official
- [BPMN 2.0 Specification](http://www.omg.org/spec/BPMN/2.0/) - OMG
- [BPMN 2.0.2](https://www.omg.org/spec/BPMN/2.0.2/About-BPMN) - OMG

### Guides
- [BPMN 2.0 Reference](https://camunda.com/bpmn/reference/) - Camunda
- [BPMN Gateway Types](https://www.visual-paradigm.com/guide/bpmn/bpmn-gateway-types/) - Visual Paradigm
- [BPMN Activities](https://processmind.com/resources/docs/bpmn-building-blocks/activities) - ProcessMind
- [BPMN Events](https://processmind.com/resources/docs/bpmn-building-blocks/intermediate-event) - ProcessMind
