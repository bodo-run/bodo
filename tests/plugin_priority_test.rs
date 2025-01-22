use async_trait::async_trait;
use bodo::{
    graph::Graph,
    plugin::{Plugin, PluginConfig, PluginManager},
    Result,
};
use std::any::Any;
use std::sync::Arc;
use tokio::sync::Mutex;

struct TestPlugin {
    name: &'static str,
    priority: i32,
    execution_order: Arc<Mutex<Vec<&'static str>>>,
}

#[async_trait]
impl Plugin for TestPlugin {
    fn name(&self) -> &'static str {
        self.name
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    async fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        self.execution_order.lock().await.push(self.name);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[tokio::test]
async fn test_plugin_priority_order() -> Result<()> {
    let execution_order = Arc::new(Mutex::new(Vec::new()));

    let mut manager = PluginManager::new();
    manager.register(Box::new(TestPlugin {
        name: "low",
        priority: 0,
        execution_order: execution_order.clone(),
    }));
    manager.register(Box::new(TestPlugin {
        name: "high",
        priority: 100,
        execution_order: execution_order.clone(),
    }));
    manager.register(Box::new(TestPlugin {
        name: "medium",
        priority: 50,
        execution_order: execution_order.clone(),
    }));

    let mut graph = Graph::new();
    manager
        .run_lifecycle(&mut graph, &PluginConfig::default())
        .await?;

    let order = execution_order.lock().await;
    assert_eq!(order.as_slice(), &["high", "medium", "low"]);

    Ok(())
}

#[tokio::test]
async fn test_plugin_same_priority() -> Result<()> {
    let execution_order = Arc::new(Mutex::new(Vec::new()));

    let mut manager = PluginManager::new();
    manager.register(Box::new(TestPlugin {
        name: "first",
        priority: 50,
        execution_order: execution_order.clone(),
    }));
    manager.register(Box::new(TestPlugin {
        name: "second",
        priority: 50,
        execution_order: execution_order.clone(),
    }));

    let mut graph = Graph::new();
    manager
        .run_lifecycle(&mut graph, &PluginConfig::default())
        .await?;

    let order = execution_order.lock().await;
    assert_eq!(order.len(), 2);
    assert!(order.contains(&"first"));
    assert!(order.contains(&"second"));

    Ok(())
}

#[tokio::test]
async fn test_plugin_default_priority() -> Result<()> {
    let execution_order = Arc::new(Mutex::new(Vec::new()));

    let mut manager = PluginManager::new();
    manager.register(Box::new(TestPlugin {
        name: "explicit_zero",
        priority: 0,
        execution_order: execution_order.clone(),
    }));
    manager.register(Box::new(TestPlugin {
        name: "default",
        priority: 0, // Using default priority
        execution_order: execution_order.clone(),
    }));

    let mut graph = Graph::new();
    manager
        .run_lifecycle(&mut graph, &PluginConfig::default())
        .await?;

    let order = execution_order.lock().await;
    assert_eq!(order.len(), 2);
    assert!(order.contains(&"explicit_zero"));
    assert!(order.contains(&"default"));

    Ok(())
}
