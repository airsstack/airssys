# [WASM-TASK-010] - Block 9: Monitoring & Observability System

**Status:** not-started  
**Added:** 2025-10-20  
**Updated:** 2025-10-20  
**Priority:** High - Integration Layer  
**Layer:** 3 - Integration  
**Block:** 9 of 11  
**Estimated Effort:** 4-5 weeks  

## Overview

Implement comprehensive monitoring and observability system with metrics collection (component-level and system-level), health monitoring integrated with SupervisorNode health checks, performance tracing (operation timing and bottleneck detection), audit logging aggregation, alerting mechanism, and metrics export (Prometheus format) achieving <10μs metrics collection overhead.

## Context

**Current State:**
- airssys-rt SupervisorNode: Health monitoring infrastructure ready
- Actor system: Message metrics available (~211ns routing)
- Component system: Lifecycle events tracked
- OSL operations: All operations audited
- Architecture: Component observability patterns defined

**Problem Statement:**
Production WASM component systems need visibility:
1. **Metrics Collection** - Component performance, resource usage
2. **Health Monitoring** - Component health status, failures
3. **Performance Tracing** - Operation timing, bottleneck detection
4. **Audit Logging** - Security-relevant events aggregated
5. **Alerting** - Proactive issue detection and notification
6. **Metrics Export** - Integration with monitoring tools (Prometheus)

Requirements:
- Real-time metrics without performance impact (<10μs overhead)
- Component-level and system-level visibility
- Health status integration with SupervisorNode
- Performance trace correlation across components
- Audit log aggregation and search
- Standard metrics export (Prometheus compatible)
- Configurable alerting rules

**Why This Block Matters:**
Without monitoring and observability:
- Production issues invisible until failure
- Performance problems undetected
- No capacity planning data
- Security events unnoticed
- Debugging requires code changes

This block enables production operations and maintenance.

## Objectives

### Primary Objective
Implement comprehensive monitoring and observability system with metrics collection (component + system level), SupervisorNode health integration, performance tracing, audit log aggregation, alerting mechanism, and Prometheus metrics export achieving <10μs collection overhead.

### Secondary Objectives
- Collect 50+ key metrics per component
- Integrate health checks with SupervisorNode
- Enable sub-millisecond performance tracing
- Aggregate audit logs with full-text search
- Support 10,000+ events/second throughput
- Prometheus-compatible metrics export

## Scope

### In Scope
1. **Metrics Collection System** - Component and system metrics
2. **Health Monitoring** - SupervisorNode integration, health checks
3. **Performance Tracing** - Operation timing, span tracing
4. **Audit Log Aggregation** - OSL audit log collection and search
5. **Alerting Mechanism** - Rule-based alerting with notifications
6. **Metrics Export** - Prometheus-compatible HTTP endpoint
7. **Observability Dashboard** - CLI-based metrics viewer

### Out of Scope
- Distributed tracing across hosts (Phase 1 single-host)
- Graphical dashboard UI (CLI only Phase 1, Web UI Phase 2)
- Log aggregation from external sources (Phase 2)
- APM tool integration (Phase 2, Prometheus only Phase 1)
- Real-user monitoring (RUM) (Phase 2)

## Implementation Plan

### Phase 1: Metrics Collection Foundation (Week 1)

#### Task 1.1: Metrics Data Model
**Deliverables:**
- Metric types (counter, gauge, histogram, summary)
- Metric naming conventions
- Label/tag system for dimensions
- Time-series data structure
- Metrics model documentation

**Success Criteria:**
- All metric types supported
- Naming conventions clear
- Labels enable slicing/dicing
- Time-series efficient
- Model well-documented

#### Task 1.2: Metrics Collection Infrastructure
**Deliverables:**
- MetricsCollector service
- Lock-free metrics storage (minimal contention)
- Metrics aggregation logic
- Metrics retention policy
- Collection infrastructure documentation

**Success Criteria:**
- Collection overhead <10μs
- Lock-free for hot paths
- Aggregation efficient
- Retention configurable
- Infrastructure scalable

#### Task 1.3: Component-Level Metrics
**Deliverables:**
- Message processing metrics (count, latency)
- Resource usage metrics (memory, CPU estimate)
- Operation metrics (filesystem, network, process counts)
- Error rate metrics (by error type)
- Component metrics documentation

**Success Criteria:**
- Metrics cover key operations
- Per-component isolation
- Overhead minimal
- Metrics useful for debugging
- Documentation comprehensive

---

### Phase 2: System-Level Metrics (Week 1-2)

#### Task 2.1: Actor System Metrics
**Deliverables:**
- MessageBroker metrics (routing latency, throughput)
- Actor lifecycle metrics (spawns, shutdowns)
- Mailbox metrics (queue depth, processing time)
- Supervision metrics (restart counts, failure types)
- Actor system metrics documentation

**Success Criteria:**
- MessageBroker performance visible
- Actor lifecycle tracked
- Mailbox backpressure detected
- Supervision events captured
- Metrics comprehensive

#### Task 2.2: WASM Runtime Metrics
**Deliverables:**
- Component instantiation metrics (time, success rate)
- Memory usage metrics (per component, total)
- CPU limiting metrics (fuel consumption, throttling)
- Compilation cache metrics (hit rate)
- Runtime metrics documentation

**Success Criteria:**
- Runtime performance visible
- Memory usage tracked
- CPU limiting effective
- Cache efficiency measured
- Metrics actionable

#### Task 2.3: Storage and Lifecycle Metrics
**Deliverables:**
- Storage operation metrics (get/set latency, throughput)
- Storage space usage metrics (per component, total)
- Component installation metrics (time, source)
- Update/rollback metrics (frequency, success rate)
- Storage/lifecycle metrics documentation

**Success Criteria:**
- Storage performance visible
- Space usage tracked
- Lifecycle operations monitored
- Update success tracked
- Metrics complete

---

### Phase 3: Health Monitoring Integration (Week 2-3)

#### Task 3.1: SupervisorNode Health Integration
**Deliverables:**
- Health check interface for components
- Health status enum (healthy, degraded, unhealthy)
- SupervisorNode health check scheduling
- Health status aggregation
- Health integration documentation

**Success Criteria:**
- Components report health status
- SupervisorNode checks health automatically
- Health status aggregated correctly
- Unhealthy components detected
- Integration seamless

#### Task 3.2: Component Health Checks
**Deliverables:**
- Default health checks (message processing, errors)
- Custom health check support (component-defined)
- Health check timeout enforcement
- Health check failure handling
- Health check documentation

**Success Criteria:**
- Default checks comprehensive
- Custom checks supported
- Timeouts prevent hangs
- Failures handled gracefully
- Checks well-documented

#### Task 3.3: System Health Dashboard
**Deliverables:**
- CLI health status viewer
- Component health summary
- System health aggregation
- Health history tracking
- Dashboard documentation

**Success Criteria:**
- Dashboard shows current health
- Component status visible
- System health aggregated
- History queryable
- Dashboard intuitive

---

### Phase 4: Performance Tracing (Week 3)

#### Task 4.1: Span Tracing Infrastructure
**Deliverables:**
- Tracing span data model (start, end, duration)
- Span context propagation (across actors)
- Parent-child span relationships
- Trace ID generation and tracking
- Tracing infrastructure documentation

**Success Criteria:**
- Spans capture operation timing
- Context propagates correctly
- Relationships tracked
- Trace IDs unique
- Infrastructure efficient

#### Task 4.2: Operation Tracing Integration
**Deliverables:**
- Host function call tracing (OSL operations)
- Message processing tracing
- Storage operation tracing
- Component lifecycle tracing
- Tracing integration documentation

**Success Criteria:**
- All operations traced
- Tracing overhead <50μs
- Traces correlatable
- Bottlenecks identifiable
- Integration comprehensive

#### Task 4.3: Trace Visualization
**Deliverables:**
- CLI trace viewer (Gantt-style)
- Trace filtering and search
- Critical path identification
- Trace export (JSON format)
- Visualization documentation

**Success Criteria:**
- Traces viewable in CLI
- Filtering works well
- Critical path highlighted
- Export enables external analysis
- Visualization useful

---

### Phase 5: Audit Logging and Alerting (Week 3-4)

#### Task 5.1: Audit Log Aggregation
**Deliverables:**
- Audit log collection from OSL
- Structured log storage (JSON Lines)
- Log indexing for search
- Log retention policy
- Audit aggregation documentation

**Success Criteria:**
- All OSL audit logs collected
- Logs structured consistently
- Search fast (indexed)
- Retention configurable
- Aggregation reliable

#### Task 5.2: Log Search and Query
**Deliverables:**
- Full-text log search
- Field-based filtering (component, operation, timestamp)
- Log export functionality
- Search API for tooling
- Search documentation

**Success Criteria:**
- Search fast and accurate
- Filtering flexible
- Export works correctly
- API usable by tools
- Search well-documented

#### Task 5.3: Alerting System
**Deliverables:**
- Alert rule configuration (threshold-based)
- Alert evaluation engine
- Alert notification channels (log, webhook)
- Alert history tracking
- Alerting system documentation

**Success Criteria:**
- Rules configurable easily
- Evaluation efficient
- Notifications reliable
- History queryable
- System comprehensive

---

### Phase 6: Metrics Export and Testing (Week 4-5)

#### Task 6.1: Prometheus Metrics Export
**Deliverables:**
- Prometheus-compatible metrics endpoint (/metrics)
- Metric format conversion (internal → Prometheus)
- Label handling for Prometheus
- HTTP server for metrics endpoint
- Prometheus export documentation

**Success Criteria:**
- Endpoint Prometheus-compatible
- Metrics formatted correctly
- Labels preserved
- HTTP server stable
- Export well-documented

#### Task 6.2: Metrics Retention and Cleanup
**Deliverables:**
- Time-series retention policy
- Old metrics cleanup background task
- Storage space management
- Retention configuration
- Retention documentation

**Success Criteria:**
- Old metrics cleaned up
- Storage space controlled
- Retention configurable
- Cleanup efficient
- Policy documented

#### Task 6.3: Comprehensive Observability Testing
**Deliverables:**
- Metrics collection tests
- Health monitoring tests
- Tracing tests
- Audit log tests
- Performance benchmarks

**Success Criteria:**
- Test coverage >90%
- All subsystems tested
- Performance validated (<10μs)
- No metrics loss detected
- Comprehensive test suite

---

## Success Criteria

### Definition of Done
This task is complete when:

1. ✅ **Metrics Collection Operational**
   - Component and system metrics collected
   - Collection overhead <10μs
   - 50+ key metrics tracked
   - Lock-free hot path

2. ✅ **Health Monitoring Integrated**
   - SupervisorNode integration working
   - Component health checks functional
   - Health status aggregated
   - Unhealthy components detected

3. ✅ **Performance Tracing Working**
   - Span tracing operational
   - Operation timing captured
   - Trace visualization available
   - Overhead <50μs per span

4. ✅ **Audit Logging Aggregated**
   - All OSL audit logs collected
   - Full-text search functional
   - Log retention managed
   - Search performance acceptable

5. ✅ **Alerting System Functional**
   - Alert rules configurable
   - Evaluation engine working
   - Notifications reliable
   - Alert history tracked

6. ✅ **Prometheus Export Ready**
   - /metrics endpoint working
   - Format Prometheus-compatible
   - Metrics scraped successfully
   - Export performant

7. ✅ **CLI Dashboard Available**
   - Metrics viewer functional
   - Health status visible
   - Trace viewer working
   - Dashboard intuitive

8. ✅ **Testing & Documentation Complete**
   - Test coverage >90%
   - Performance validated
   - Complete observability guide
   - Integration examples

## Dependencies

### Upstream Dependencies
- ✅ airssys-rt SupervisorNode: Health monitoring infrastructure
- ✅ WASM-TASK-004: Actor System Integration (Block 3) - **REQUIRED** for actor metrics
- ✅ WASM-TASK-009: OSL Bridge (Block 8) - **REQUIRED** for audit log collection
- ✅ WASM-TASK-007: Persistent Storage (Block 6) - **REQUIRED** for metrics storage

### Downstream Dependencies (Blocks This Task)
- WASM-TASK-012: CLI Tool (Block 11) - needs observability commands (logs, metrics, health)
- Production operations - depends on monitoring for reliability

### External Dependencies
- prometheus client library (metrics format)
- serde_json (structured logging)
- chrono (timestamps)

## Risks and Mitigations

### Risk 1: Metrics Collection Performance Impact
**Impact:** High - Slow metrics collection degrades system  
**Probability:** Medium - Naive collection adds overhead  
**Mitigation:**
- Lock-free data structures for hot paths
- Sampling for high-frequency events
- Async aggregation off hot path
- Continuous performance benchmarking

### Risk 2: Metrics Storage Growth
**Impact:** Medium - Unbounded storage growth  
**Probability:** High - Without retention, storage grows  
**Mitigation:**
- Retention policy mandatory
- Automatic cleanup background task
- Storage space monitoring
- Configurable retention period

### Risk 3: Health Check False Positives
**Impact:** Medium - False unhealthy status causes restarts  
**Probability:** Medium - Health checks can be flaky  
**Mitigation:**
- Configurable health check thresholds
- Multiple failures before unhealthy
- Health check timeouts generous
- Clear health check debugging

### Risk 4: Audit Log Volume
**Impact:** Medium - High volume audit logs slow search  
**Probability:** High - OSL operations generate many logs  
**Mitigation:**
- Indexed log storage (fast search)
- Log level filtering
- Retention policy for old logs
- Log sampling in high-volume scenarios

### Risk 5: Alerting Noise
**Impact:** Low - Too many alerts ignored  
**Probability:** High - Naive alerting generates noise  
**Mitigation:**
- Alert rule testing before deployment
- Alert throttling (don't repeat too often)
- Alert severity levels
- Clear alert documentation

## Progress Tracking

**Overall Status:** not-started - 0%

### Phase Breakdown
| Phase | Description | Status | Estimated Duration | Notes |
|-------|-------------|--------|-------------------|-------|
| 1 | Metrics Collection Foundation | not-started | Week 1 | Foundation |
| 2 | System-Level Metrics | not-started | Week 1-2 | Comprehensive metrics |
| 3 | Health Monitoring Integration | not-started | Week 2-3 | SupervisorNode integration |
| 4 | Performance Tracing | not-started | Week 3 | Tracing system |
| 5 | Audit Logging and Alerting | not-started | Week 3-4 | Logs & alerts |
| 6 | Metrics Export and Testing | not-started | Week 4-5 | Export & QA |

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | Metrics Data Model | not-started | - | Foundation |
| 1.2 | Metrics Collection Infrastructure | not-started | - | Collection system |
| 1.3 | Component-Level Metrics | not-started | - | Component metrics |
| 2.1 | Actor System Metrics | not-started | - | Actor metrics |
| 2.2 | WASM Runtime Metrics | not-started | - | Runtime metrics |
| 2.3 | Storage and Lifecycle Metrics | not-started | - | Storage metrics |
| 3.1 | SupervisorNode Health Integration | not-started | - | Health checks |
| 3.2 | Component Health Checks | not-started | - | Health interface |
| 3.3 | System Health Dashboard | not-started | - | Health UI |
| 4.1 | Span Tracing Infrastructure | not-started | - | Tracing foundation |
| 4.2 | Operation Tracing Integration | not-started | - | Tracing integration |
| 4.3 | Trace Visualization | not-started | - | Trace viewer |
| 5.1 | Audit Log Aggregation | not-started | - | Log collection |
| 5.2 | Log Search and Query | not-started | - | Log search |
| 5.3 | Alerting System | not-started | - | Alerts |
| 6.1 | Prometheus Metrics Export | not-started | - | Prometheus |
| 6.2 | Metrics Retention and Cleanup | not-started | - | Retention |
| 6.3 | Comprehensive Observability Testing | not-started | - | Quality assurance |

## Progress Log

*No progress yet - task just created*

## Related Documentation

### ADRs
- **ADR-WASM-011: Observability Architecture** - (Future) Metrics and monitoring design

### Knowledge Documentation
- **KNOWLEDGE-RT-015: SupervisorNode Monitoring Integration** - Health check patterns
- **KNOWLEDGE-RT-005: MessageBroker Performance Analysis** - MessageBroker metrics
- **KNOWLEDGE-WASM-011: OSL Bridge Design** - Audit log integration

### External References
- [Prometheus Exposition Format](https://prometheus.io/docs/instrumenting/exposition_formats/)
- [OpenTelemetry](https://opentelemetry.io/) (Phase 2 consideration)
- [Structured Logging Best Practices](https://www.thoughtworks.com/insights/blog/structured-logging)

## Notes

**Metrics Collection Overhead Critical:**
Target <10μs overhead per metric event.
Achieved through:
- Lock-free atomic operations for counters/gauges
- Batched histogram updates
- Async aggregation off hot path
- Sampling for ultra-high-frequency events

**Lock-Free Metrics Pattern:**
```rust
// Hot path: increment counter (lock-free atomic)
metrics.counter("messages_processed").increment();

// Background: aggregate and export (not on hot path)
tokio::spawn(async move {
    metrics.aggregate_and_export().await;
});
```

**Health Check Integration:**
SupervisorNode already has health monitoring infrastructure.
Components implement health check interface:
```rust
trait ComponentHealth {
    fn health_status(&self) -> HealthStatus;
}

enum HealthStatus {
    Healthy,      // Operating normally
    Degraded,     // Functioning but with issues
    Unhealthy,    // Not functioning, needs restart
}
```

**Span Tracing for Performance:**
Tracing captures operation timing:
- Host function calls (OSL operations)
- Message processing
- Storage operations
- Component lifecycle events

Traces correlate across components via trace ID.
Critical path analysis identifies bottlenecks.

**Audit Log Aggregation:**
OSL operations already audited at source.
Observability system aggregates logs for:
- Central search and query
- Security event analysis
- Compliance reporting
- Forensic investigation

Structured JSON Lines format enables efficient indexing.

**Alerting Rule Examples:**
```yaml
alerts:
  - name: high_error_rate
    condition: error_rate > 10 per minute
    severity: warning
    notify: log, webhook
  
  - name: component_unhealthy
    condition: health_status == unhealthy
    severity: critical
    notify: log, webhook
  
  - name: high_memory_usage
    condition: memory_usage > 80%
    severity: warning
    notify: log
```

**Prometheus Integration:**
Export metrics via HTTP endpoint `/metrics`:
```
# HELP component_messages_processed Total messages processed by component
# TYPE component_messages_processed counter
component_messages_processed{component="example"} 12345

# HELP component_processing_latency_seconds Message processing latency
# TYPE component_processing_latency_seconds histogram
component_processing_latency_seconds_bucket{component="example",le="0.001"} 100
component_processing_latency_seconds_bucket{component="example",le="0.01"} 450
component_processing_latency_seconds_sum{component="example"} 5.2
component_processing_latency_seconds_count{component="example"} 500
```

**CLI Dashboard Design:**
Terminal-based metrics viewer:
- Real-time metrics updates
- Component health status grid
- Top components by metric (CPU, memory, errors)
- Trace viewer (Gantt-style spans)
- Log search interface

**Metrics Retention Strategy:**
- Raw metrics: 24 hours (full resolution)
- Aggregated hourly: 7 days
- Aggregated daily: 30 days
- Configurable per deployment

**Phase 2 Enhancements:**
- Web-based dashboard UI (Grafana integration)
- Distributed tracing across hosts (OpenTelemetry)
- APM tool integration (Datadog, New Relic)
- Real-user monitoring (RUM) for component UX
- Log aggregation from external sources
- Machine learning anomaly detection
