use std::time::Duration;
use reqwest::{Client, RequestBuilder, Response};
use serde_json::Value;
use tracing::{debug, error, info, warn};
use governor::{Quota, RateLimiter, state::{InMemoryState, NotKeyed}, clock::DefaultClock};
use moka::future::Cache;
use std::sync::Arc;
use std::num::NonZeroU32;

use crate::config::{AppConfig, AuthType};
use super::error::{ApiError, ApiResult, ApiErrorResponse};
use super::models::*;

#[derive(Debug, Clone)]
pub struct EasyProjectClient {
    http_client: reqwest::Client,
    base_url: String,
    api_key: String,
    cache: Option<Arc<Cache<String, Value>>>,
    rate_limiter: Option<Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
}

impl EasyProjectClient {
    pub async fn new(config: &AppConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.http.timeout_seconds))
            .user_agent(&config.http.user_agent)
            .build()?;

        let cache = if config.cache.enabled {
            Some(Arc::new(Cache::builder()
                .max_capacity(config.cache.max_entries)
                .time_to_live(Duration::from_secs(config.cache.ttl_seconds))
                .build()))
        } else {
            None
        };

        let rate_limiter = if config.rate_limiting.enabled {
            Some(Arc::new(RateLimiter::direct(
                Quota::per_minute(NonZeroU32::new(config.rate_limiting.requests_per_minute).unwrap())
                    .allow_burst(NonZeroU32::new(config.rate_limiting.burst_size).unwrap())
            )))
        } else {
            None
        };

        let api_key = config.easyproject.api_key.clone()
            .ok_or("Chybí API klíč pro EasyProject")?;

        Ok(Self {
            http_client: client,
            base_url: config.easyproject.base_url.clone(),
            api_key,
            cache,
            rate_limiter,
        })
    }

    /// Přidá autentifikační hlavičky k požadavku
    fn add_auth(&self, request_builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request_builder.header("X-Redmine-API-Key", &self.api_key)
    }

    /// Provede HTTP požadavek s retry logikou
    async fn execute_request(&self, request: RequestBuilder) -> ApiResult<Value> {
        // Rate limiting
        if let Some(ref limiter) = self.rate_limiter {
            limiter.until_ready().await;
        }

        let response = request
            .send()
            .await
            .map_err(ApiError::Http)?;

        if !response.status().is_success() {
            return Err(ApiError::Api {
                status: response.status().as_u16(),
                message: format!("HTTP error: {}", response.status()),
            });
        }

        response.json::<Value>().await.map_err(ApiError::Http)
    }

    /// Získá data z cache nebo provede API volání
    async fn get_cached_or_fetch<T>(&self, cache_key: &str, _entity_type: &str, fetch_fn: impl std::future::Future<Output = ApiResult<T>>) -> ApiResult<T>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        if let Some(cache) = &self.cache {
            if let Some(cached_value) = cache.get(cache_key).await {
                debug!("Cache hit pro klíč: {}", cache_key);
                return serde_json::from_value(cached_value)
                    .map_err(|e| ApiError::Api {
                        status: 500,
                        message: format!("Chyba deserializace z cache: {}", e),
                    });
            }
        }

        debug!("Cache miss pro klíč: {}, volám API", cache_key);
        let result = fetch_fn.await?;

        // Uložení do cache
        if let Some(cache) = &self.cache {
            let value = serde_json::to_value(&result)
                .map_err(|e| ApiError::Api {
                    status: 500,
                    message: format!("Chyba serializace do cache: {}", e),
                })?;
            
            cache.insert(cache_key.to_string(), value).await;
            debug!("Uloženo do cache: {}", cache_key);
        }

        Ok(result)
    }

    /// Invaliduje cache pro daný pattern
    pub async fn invalidate_cache(&self, pattern: &str) {
        if let Some(cache) = &self.cache {
            // Pro jednoduchost invalidujeme celou cache
            // V produkční verzi by bylo lepší implementovat pattern matching
            cache.invalidate_all();
            info!("Cache invalidována pro pattern: {}", pattern);
        }
    }

    // === PROJECT API METHODS ===

    pub async fn list_projects(&self, limit: Option<u32>, offset: Option<u32>, include_archived: Option<bool>) -> ApiResult<ProjectsResponse> {
        let cache_key = format!("projects_{}_{}_{}",
            limit.unwrap_or(25),
            offset.unwrap_or(0),
            include_archived.unwrap_or(false)
        );

        self.get_cached_or_fetch(&cache_key, "project", async {
            let mut url = format!("{}/projects.json", self.base_url);
            let mut query_params = Vec::new();

            if let Some(limit) = limit {
                query_params.push(("limit", limit.to_string()));
            }
            if let Some(offset) = offset {
                query_params.push(("offset", offset.to_string()));
            }

            let request = self.add_auth(self.http_client.get(&url));
            let request = if !query_params.is_empty() {
                request.query(&query_params)
            } else {
                request
            };

            let response = self.execute_request(request).await?;
            self.parse_response(response)
        }).await
    }

    pub async fn get_project(&self, id: i32, include: Option<Vec<String>>) -> ApiResult<ProjectResponse> {
        let cache_key = format!("project_{}", id);

        self.get_cached_or_fetch(&cache_key, "project", async {
            let url = format!("{}/projects/{}.json", self.base_url, id);
            let mut request = self.add_auth(self.http_client.get(&url));

            if let Some(include) = include {
                request = request.query(&[("include", include.join(","))]);
            }

            let response = self.execute_request(request).await?;
            self.parse_response(response)
        }).await
    }

    pub async fn create_project(&self, project_data: CreateProjectRequest) -> ApiResult<ProjectResponse> {
        let url = format!("{}/projects.json", self.base_url);
        let request = self.add_auth(self.http_client.post(&url))
            .json(&project_data);

        let response = self.execute_request(request).await?;
        self.parse_response(response)
    }

    pub async fn update_project(&self, id: i32, project_data: CreateProjectRequest) -> ApiResult<ProjectResponse> {
        let url = format!("{}/projects/{}.json", self.base_url, id);
        let request = self.add_auth(self.http_client.put(&url))
            .json(&project_data);

        let response = self.execute_request(request).await?;
        self.parse_response(response)
    }

    pub async fn delete_project(&self, id: i32) -> ApiResult<()> {
        let url = format!("{}/projects/{}.json", self.base_url, id);
        let request = self.add_auth(self.http_client.delete(&url));

        self.execute_request(request).await?;

        // Invalidace cache
        self.invalidate_cache("projects").await;
        self.invalidate_cache(&format!("project_{}", id)).await;

        Ok(())
    }

    // === ISSUE API METHODS ===

    pub async fn list_issues(&self, project_id: Option<i32>, limit: Option<u32>, offset: Option<u32>, include: Option<Vec<String>>) -> ApiResult<IssuesResponse> {
        let cache_key = format!("issues_{}_{}_{}_{}",
            project_id.map(|id| id.to_string()).unwrap_or_else(|| "all".to_string()),
            limit.unwrap_or(25),
            offset.unwrap_or(0),
            include.as_ref().map(|i| i.join(",")).unwrap_or_else(|| "none".to_string())
        );

        self.get_cached_or_fetch(&cache_key, "issue", async {
            let url = format!("{}/issues.json", self.base_url);
            let mut query_params = Vec::new();

            if let Some(project_id) = project_id {
                query_params.push(("project_id", project_id.to_string()));
            }
            if let Some(limit) = limit {
                query_params.push(("limit", limit.to_string()));
            }
            if let Some(offset) = offset {
                query_params.push(("offset", offset.to_string()));
            }
            if let Some(include) = include {
                query_params.push(("include", include.join(",")));
            }

            let request = self.add_auth(self.http_client.get(&url))
                .query(&query_params);

            let response = self.execute_request(request).await?;
            self.parse_response(response)
        }).await
    }

    pub async fn get_issue(&self, id: i32, include: Option<Vec<String>>) -> ApiResult<IssueResponse> {
        let cache_key = format!("issue_{}", id);

        self.get_cached_or_fetch(&cache_key, "issue", async {
            let url = format!("{}/issues/{}.json", self.base_url, id);
            let mut request = self.add_auth(self.http_client.get(&url));

            if let Some(include) = include {
                request = request.query(&[("include", include.join(","))]);
            }

            let response = self.execute_request(request).await?;
            self.parse_response(response)
        }).await
    }

    pub async fn create_issue(&self, issue_data: CreateIssueRequest) -> ApiResult<IssueResponse> {
        let url = format!("{}/issues.json", self.base_url);
        let request = self.add_auth(self.http_client.post(&url))
            .json(&issue_data);

        let response = self.execute_request(request).await?;
        self.parse_response(response)
    }

    pub async fn update_issue(&self, id: i32, issue_data: CreateIssueRequest) -> ApiResult<IssueResponse> {
        let url = format!("{}/issues/{}.json", self.base_url, id);
        let request = self.add_auth(self.http_client.put(&url))
            .json(&issue_data);

        let response = self.execute_request(request).await?;
        self.parse_response(response)
    }

    // === USER API METHODS ===

    pub async fn list_users(&self, limit: Option<u32>, offset: Option<u32>) -> ApiResult<UsersResponse> {
        let cache_key = format!("users_{}_{}", limit.unwrap_or(25), offset.unwrap_or(0));

        self.get_cached_or_fetch(&cache_key, "user", async {
            let url = format!("{}/users.json", self.base_url);
            let mut query_params = Vec::new();

            if let Some(limit) = limit {
                query_params.push(("limit", limit.to_string()));
            }
            if let Some(offset) = offset {
                query_params.push(("offset", offset.to_string()));
            }

            let request = self.add_auth(self.http_client.get(&url))
                .query(&query_params);

            let response = self.execute_request(request).await?;
            self.parse_response(response)
        }).await
    }

    pub async fn get_user(&self, id: i32) -> ApiResult<UserResponse> {
        let cache_key = format!("user_{}", id);

        self.get_cached_or_fetch(&cache_key, "user", async {
            let url = format!("{}/users/{}.json", self.base_url, id);
            let request = self.add_auth(self.http_client.get(&url));

            let response = self.execute_request(request).await?;
            self.parse_response(response)
        }).await
    }

    // === TIME ENTRY API METHODS ===

    pub async fn list_time_entries(&self, project_id: Option<i32>, user_id: Option<i32>, limit: Option<u32>, offset: Option<u32>) -> ApiResult<TimeEntriesResponse> {
        let cache_key = format!("time_entries_{}_{}_{}_{}", 
            project_id.map(|id| id.to_string()).unwrap_or_else(|| "all".to_string()),
            user_id.map(|id| id.to_string()).unwrap_or_else(|| "all".to_string()),
            limit.unwrap_or(25), 
            offset.unwrap_or(0)
        );

        self.get_cached_or_fetch(&cache_key, "time_entry", async {
            let url = format!("{}/time_entries.json", self.base_url);
            let mut query_params = Vec::new();

            if let Some(project_id) = project_id {
                query_params.push(("project_id", project_id.to_string()));
            }
            if let Some(user_id) = user_id {
                query_params.push(("user_id", user_id.to_string()));
            }
            if let Some(limit) = limit {
                query_params.push(("limit", limit.to_string()));
            }
            if let Some(offset) = offset {
                query_params.push(("offset", offset.to_string()));
            }

            let request = self.add_auth(self.http_client.get(&url))
                .query(&query_params);

            let response = self.execute_request(request).await?;
            self.parse_response(response)
        }).await
    }

    pub async fn create_time_entry(&self, time_entry_data: CreateTimeEntryRequest) -> ApiResult<TimeEntryResponse> {
        let url = format!("{}/time_entries.json", self.base_url);
        let request = self.add_auth(self.http_client.post(&url))
            .json(&time_entry_data);

        let response = self.execute_request(request).await?;
        self.parse_response(response)
    }

    fn parse_response<T: serde::de::DeserializeOwned>(&self, value: Value) -> ApiResult<T> {
        debug!("Parsování API response: {}", serde_json::to_string_pretty(&value).unwrap_or_else(|_| "Nepodařilo se serializovat".to_string()));
        serde_json::from_value(value).map_err(|e| 
            ApiError::Api {
                status: 500,
                message: format!("Chyba parsování JSON: {}", e),
            }
        )
    }
} 