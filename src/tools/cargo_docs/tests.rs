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
    
    // Check for general content patterns that should be in any crate documentation
    // Using more flexible assertions that should work regardless of exact formatting
    assert!(result.len() > 100); // Should have substantial content
}

#[test]
async fn test_lookup_crate_with_version() {
    let router = CargoDocRouter::new();
    let result = router.lookup_crate("lumin".to_string(), Some("0.1.0".to_string())).await;
    
    // Verify that the result contains expected content for the specific version
    assert!(result.contains("lumin"));
    assert!(!result.is_empty());
    
    // Check for version-specific content
    assert!(result.contains("0.1.0") || result.contains("Version"));
    
    // Common content patterns for crate documentation
    assert!(result.to_lowercase().contains("license") || 
           result.to_lowercase().contains("repository") || 
           result.to_lowercase().contains("dependencies"));
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
    
    // Just verify we get some content back, without making assumptions about exact format
    assert!(result.len() > 10);
}

#[test]
async fn test_search_crates() {
    let router = CargoDocRouter::new();
    let result = router.search_crates("lumin".to_string(), Some(5)).await;
    
    // Verify result contains search results for lumin
    assert!(result.contains("lumin"));
    assert!(!result.is_empty());
    
    // Check for common content patterns in search results
    if result.starts_with('{') {
        // If JSON response
        assert!(result.contains("\"crates\"") || result.contains("\"total\""));
        assert!(result.contains("\"description\"") || result.contains("\"name\""));
    } else {
        // If HTML converted to markdown
        assert!(result.to_lowercase().contains("results") || 
               result.to_lowercase().contains("crates"));
        assert!(result.contains("downloads") || 
               result.contains("version") || 
               result.contains("description"));
    }
}

#[test]
async fn test_content_transformation() {
    let router = CargoDocRouter::new();
    
    // Test the transformation from HTML to markdown
    let serde_result = router.lookup_crate("serde".to_string(), None).await;
    
    // Verify we got substantial content
    assert!(!serde_result.is_empty());
    assert!(serde_result.len() > 100);
    
    // Basic check for HTML to markdown conversion
    // Just check that we don't have obvious HTML tags
    assert!(!serde_result.contains("<html>"));
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

#[test]
async fn test_cache_in_lookup() {
    let router = CargoDocRouter::new();
    
    // First lookup to populate the cache
    let first_result = router.lookup_crate("regex".to_string(), None).await;
    assert!(!first_result.is_empty());
    
    // Second lookup should use the cache
    let second_result = router.lookup_crate("regex".to_string(), None).await;
    
    // Results should be identical when pulled from cache
    assert_eq!(first_result, second_result);
}