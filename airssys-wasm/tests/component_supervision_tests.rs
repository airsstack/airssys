#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Integration tests for component supervision

#[cfg(test)]
mod component_supervision_tests {
    use airssys_wasm::actor::{
        BackoffStrategy, ComponentSupervisor, RestartPolicy, SupervisionState, SupervisorConfig,
    };
    use airssys_wasm::core::ComponentId;
    use chrono::Utc;
    use std::time::Duration;

    // ============================================================================
    // Restart Policy Tests
    // ============================================================================

    #[test]
    fn test_supervise_component_with_permanent_policy() {
        let mut supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");
        let supervisor_config = SupervisorConfig::permanent();

        let result = supervisor.supervise(&component_id, supervisor_config);
        assert!(result.is_ok());

        let handle = supervisor.get_handle(&component_id);
        assert!(handle.is_some());
        if let Some(h) = handle {
            assert_eq!(h.state, SupervisionState::Initializing);
        }
    }

    #[test]
    fn test_supervise_component_with_transient_policy() {
        let mut supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");
        let supervisor_config = SupervisorConfig::transient();

        let result = supervisor.supervise(&component_id, supervisor_config);
        assert!(result.is_ok());

        let handle = supervisor.get_handle(&component_id);
        assert!(handle.is_some());
        if let Some(h) = handle {
            assert_eq!(h.restart_count, 0);
        }
    }

    #[test]
    fn test_supervise_component_with_temporary_policy() {
        let mut supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");
        let supervisor_config = SupervisorConfig::temporary();

        let result = supervisor.supervise(&component_id, supervisor_config);
        assert!(result.is_ok());
    }

    // ============================================================================
    // Restart Decision Tests
    // ============================================================================

    #[test]
    fn test_restart_policy_decision_permanent() {
        let policy = RestartPolicy::Permanent;
        assert!(policy.should_restart(true)); // Restart on error
        assert!(policy.should_restart(false)); // Restart on normal exit
    }

    #[test]
    fn test_restart_policy_decision_transient() {
        let policy = RestartPolicy::Transient;
        assert!(policy.should_restart(true)); // Restart on error
        assert!(!policy.should_restart(false)); // Don't restart on normal exit
    }

    #[test]
    fn test_restart_policy_decision_temporary() {
        let policy = RestartPolicy::Temporary;
        assert!(!policy.should_restart(true)); // Never restart
        assert!(!policy.should_restart(false)); // Never restart
    }

    // ============================================================================
    // Backoff Strategy Tests
    // ============================================================================

    #[test]
    fn test_backoff_immediate_strategy() {
        let strategy = BackoffStrategy::Immediate;
        assert_eq!(strategy.calculate_delay(1), Duration::from_millis(0));
        assert_eq!(strategy.calculate_delay(10), Duration::from_millis(0));
    }

    #[test]
    fn test_backoff_linear_strategy() {
        let strategy = BackoffStrategy::Linear {
            base_delay: Duration::from_millis(100),
        };
        assert_eq!(strategy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(strategy.calculate_delay(2), Duration::from_millis(200));
        assert_eq!(strategy.calculate_delay(5), Duration::from_millis(500));
    }

    #[test]
    fn test_backoff_exponential_strategy() {
        let strategy = BackoffStrategy::Exponential {
            base_delay: Duration::from_millis(100),
            multiplier: 2.0,
            max_delay: Duration::from_secs(30),
        };
        // 100 * 2^0 = 100
        assert_eq!(strategy.calculate_delay(0), Duration::from_millis(100));
        // 100 * 2^1 = 200
        assert_eq!(strategy.calculate_delay(1), Duration::from_millis(200));
        // 100 * 2^2 = 400
        assert_eq!(strategy.calculate_delay(2), Duration::from_millis(400));
        // 100 * 2^8 = 25600
        assert_eq!(strategy.calculate_delay(8), Duration::from_millis(25600));
        // 100 * 2^9 = 51200, capped at 30000
        assert_eq!(strategy.calculate_delay(9), Duration::from_secs(30));
    }

    // ============================================================================
    // Restart Limit Tests
    // ============================================================================

    #[test]
    fn test_max_restart_limit_check() {
        let config = SupervisorConfig {
            max_restarts: 3,
            time_window: Duration::from_secs(60),
            ..Default::default()
        };

        let now = Utc::now();

        // Within time window - should trigger limit
        let recent = vec![
            (now - Duration::from_secs(30), true),
            (now - Duration::from_secs(20), true),
            (now - Duration::from_secs(10), true),
        ];
        assert!(config.check_restart_limit(&recent));

        // Outside time window - should not trigger limit
        let old = vec![
            (now - Duration::from_secs(90), true),
            (now - Duration::from_secs(80), true),
            (now - Duration::from_secs(70), true),
        ];
        assert!(!config.check_restart_limit(&old));
    }

    // ============================================================================
    // Supervision Configuration Tests
    // ============================================================================

    #[test]
    fn test_supervision_config_builder_pattern() {
        let config = SupervisorConfig::permanent()
            .with_max_restarts(5)
            .with_time_window(Duration::from_secs(120))
            .with_shutdown_timeout(Duration::from_secs(10));

        assert_eq!(config.restart_policy, RestartPolicy::Permanent);
        assert_eq!(config.max_restarts, 5);
        assert_eq!(config.time_window, Duration::from_secs(120));
        assert_eq!(config.shutdown_timeout, Duration::from_secs(10));
    }

    // ============================================================================
    // Supervision State Transitions
    // ============================================================================

    #[tokio::test]
    async fn test_component_supervisor_state_transitions() {
        let mut supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");
        let config = SupervisorConfig::permanent();

        supervisor.supervise(&component_id, config).ok();

        // Mark as running
        supervisor.mark_running(&component_id).ok();
        if let Some(handle) = supervisor.get_handle(&component_id) {
            assert_eq!(handle.state, SupervisionState::Running);
        }

        // Mark as restarting
        supervisor.mark_restarting(&component_id).ok();
        if let Some(handle) = supervisor.get_handle(&component_id) {
            assert_eq!(handle.state, SupervisionState::Restarting);
        }
    }

    // ============================================================================
    // Supervision Tree Tests
    // ============================================================================

    #[test]
    fn test_supervision_tree_parent_child() {
        let mut supervisor = ComponentSupervisor::new();

        let parent_id = ComponentId::new("parent");
        let child_id = ComponentId::new("child");

        supervisor
            .supervise(&parent_id, SupervisorConfig::default())
            .ok();
        supervisor
            .supervise(&child_id, SupervisorConfig::default())
            .ok();

        supervisor.set_parent(&child_id, parent_id.clone()).ok();

        if let Some(child_handle) = supervisor.get_handle(&child_id) {
            assert_eq!(child_handle.parent_id.as_ref(), Some(&parent_id));
        }
    }

    #[test]
    fn test_supervision_tree_get_children() {
        let mut supervisor = ComponentSupervisor::new();

        let parent_id = ComponentId::new("parent");
        let child1_id = ComponentId::new("child1");
        let child2_id = ComponentId::new("child2");

        supervisor
            .supervise(&parent_id, SupervisorConfig::default())
            .ok();
        supervisor
            .supervise(&child1_id, SupervisorConfig::default())
            .ok();
        supervisor
            .supervise(&child2_id, SupervisorConfig::default())
            .ok();

        supervisor.set_parent(&child1_id, parent_id.clone()).ok();
        supervisor.set_parent(&child2_id, parent_id.clone()).ok();

        let children = supervisor.get_children(&parent_id);
        assert_eq!(children.len(), 2);
        assert!(children.contains(&child1_id));
        assert!(children.contains(&child2_id));
    }

    #[test]
    fn test_supervision_tree_get_ancestors() {
        let mut supervisor = ComponentSupervisor::new();

        let root_id = ComponentId::new("root");
        let parent_id = ComponentId::new("parent");
        let child_id = ComponentId::new("child");

        supervisor
            .supervise(&root_id, SupervisorConfig::default())
            .ok();
        supervisor
            .supervise(&parent_id, SupervisorConfig::default())
            .ok();
        supervisor
            .supervise(&child_id, SupervisorConfig::default())
            .ok();

        supervisor.set_parent(&parent_id, root_id.clone()).ok();
        supervisor.set_parent(&child_id, parent_id.clone()).ok();

        let ancestors = supervisor.get_ancestors(&child_id);
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0], parent_id);
        assert_eq!(ancestors[1], root_id);
    }

    #[test]
    fn test_supervision_tree_structure() {
        let mut supervisor = ComponentSupervisor::new();

        let root_id = ComponentId::new("root");
        let child1_id = ComponentId::new("child1");
        let child2_id = ComponentId::new("child2");

        supervisor
            .supervise(&root_id, SupervisorConfig::default())
            .ok();
        supervisor
            .supervise(&child1_id, SupervisorConfig::default())
            .ok();
        supervisor
            .supervise(&child2_id, SupervisorConfig::default())
            .ok();

        supervisor.set_parent(&child1_id, root_id.clone()).ok();
        supervisor.set_parent(&child2_id, root_id.clone()).ok();

        // Verify children are properly linked
        let children = supervisor.get_children(&root_id);
        assert_eq!(children.len(), 2);
        assert!(children.contains(&child1_id));
        assert!(children.contains(&child2_id));
    }

    // ============================================================================
    // Supervision Statistics Tests
    // ============================================================================

    #[test]
    fn test_supervision_statistics_empty() {
        let supervisor = ComponentSupervisor::new();
        let stats = supervisor.get_statistics();

        assert_eq!(stats.total_supervised, 0);
        assert_eq!(stats.currently_running, 0);
        assert_eq!(stats.failed_components, 0);
        assert_eq!(stats.total_restart_attempts, 0);
    }

    #[test]
    fn test_supervision_statistics_with_components() {
        let mut supervisor = ComponentSupervisor::new();

        for i in 0..5 {
            let component_id = ComponentId::new(format!("component-{}", i));
            supervisor
                .supervise(&component_id, SupervisorConfig::default())
                .ok();
        }

        let stats = supervisor.get_statistics();
        assert_eq!(stats.total_supervised, 5);
        assert_eq!(stats.currently_running, 0); // All still initializing
    }

    #[test]
    fn test_supervision_statistics_running_components() {
        let mut supervisor = ComponentSupervisor::new();

        for i in 0..3 {
            let component_id = ComponentId::new(format!("component-{}", i));
            supervisor
                .supervise(&component_id, SupervisorConfig::default())
                .ok();
            supervisor.mark_running(&component_id).ok();
        }

        let stats = supervisor.get_statistics();
        assert_eq!(stats.total_supervised, 3);
        assert_eq!(stats.currently_running, 3);
    }
}
