use super::*;
use tokio::test;

#[test]
async fn test_lookup_crate() {
    let router = CargoDocRouter::new();
    let result = router.lookup_crate("lumin".to_string(), None).await;
    
    // Verify that the result contains expected content for the lumin crate
    assert!(result.contains("lumin"));
    assert!(!result.is_empty());
    assert!(result.len() > 100); // Should have substantial content
}

#[test]
async fn test_lookup_crate_with_version() {
    let router = CargoDocRouter::new();
    let result = router.lookup_crate("lumin".to_string(), Some("0.1.0".to_string())).await;
    
    // Verify that the result contains expected content for the specific version
    assert!(result.contains("lumin"));
    assert!(!result.is_empty());
}

#[test]
async fn test_lookup_item() {
    let router = CargoDocRouter::new();
    let result = router.lookup_item_tool(
        "lumin".to_string(),
        "core::Lumin".to_string(),
        None,
    ).await;
    
    // Verify result contains the Lumin struct documentation
    assert!(!result.is_empty());
}

#[test]
async fn test_search_crates() {
    let router = CargoDocRouter::new();
    let result = router.search_crates("lumin".to_string(), Some(5)).await;
    
    // Verify result contains search results for lumin
    assert!(result.contains("lumin"));
    assert!(!result.is_empty());
}

#[test]
async fn test_doc_cache() {
    let cache = DocCache::new();
    let key = "test_key";
    let value = "test_value".to_string();
    
    // Initially the key should not exist
    assert_eq!(cache.get(key).await, None);
    
    // Set a value
    cache.set(key.to_string(), value.clone()).await;
    
    // Now we should get the value back
    assert_eq!(cache.get(key).await, Some(value));
}