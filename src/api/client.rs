use std::time::Duration;
use reqwest::{Client, RequestBuilder};
use serde_json::Value;
use tracing::{debug, info};
use governor::{Quota, RateLimiter, state::{InMemoryState, NotKeyed}, clock::DefaultClock};
use moka::future::Cache;
use std::sync::Arc;
use std::num::NonZeroU32;

use crate::config::AppConfig;
use super::error::{ApiError, ApiResult};
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

        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Neznámá chyba".to_string());
            return Err(ApiError::Api {
                status: status.as_u16(),
                message: format!("HTTP error {}: {}", status, error_text),
            });
        }

        // Zkontrolujeme, zda odpověď obsahuje data
        let response_text = response.text().await.map_err(ApiError::Http)?;
        
        if response_text.trim().is_empty() {
            // Prázdná odpověď - vrátíme prázdný objekt
            debug!("API vrátilo prázdnou odpověď");
            return Ok(serde_json::json!({}));
        }

        // Pokusíme se parsovat JSON
        serde_json::from_str(&response_text).map_err(|e| {
            debug!("Chyba parsování JSON: {}. Response text: {}", e, response_text);
            ApiError::Api {
                status: 500,
                message: format!("Chyba parsování JSON: {}. Response: {}", e, response_text),
            }
        })
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

    pub async fn list_projects(&self, limit: Option<u32>, offset: Option<u32>, include_archived: Option<bool>, easy_query_q: Option<String>, set_filter: Option<bool>, sort: Option<String>) -> ApiResult<ProjectsResponse> {
        let cache_key = format!("projects_{}_{}_{}_{}_{}_{}",
            limit.unwrap_or(25),
            offset.unwrap_or(0),
            include_archived.unwrap_or(false),
            easy_query_q.as_ref().unwrap_or(&"".to_string()),
            set_filter.unwrap_or(false),
            sort.as_ref().unwrap_or(&"".to_string())
        );

        self.get_cached_or_fetch(&cache_key, "project", async {
            let url = format!("{}/projects.json", self.base_url);
            let mut query_params = Vec::new();

            if let Some(limit) = limit {
                query_params.push(("limit", limit.to_string()));
            }
            if let Some(offset) = offset {
                query_params.push(("offset", offset.to_string()));
            }
            if let Some(query) = easy_query_q {
                query_params.push(("easy_query_q", query));
                // Pokud je easy_query_q zadáno, automaticky aktivujeme set_filter
                query_params.push(("set_filter", "1".to_string()));
            } else if let Some(true) = set_filter {
                query_params.push(("set_filter", "1".to_string()));
            }
            if let Some(sort) = sort {
                query_params.push(("sort", sort));
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

    pub async fn list_issues(&self, project_id: Option<i32>, limit: Option<u32>, offset: Option<u32>, include: Option<Vec<String>>, easy_query_q: Option<String>, set_filter: Option<bool>, sort: Option<String>, assigned_to_id: Option<i32>, status_id: Option<i32>, tracker_id: Option<i32>, priority_id: Option<i32>) -> ApiResult<IssuesResponse> {
        let cache_key = format!("issues_{}_{}_{}_{}_{}_{}_{}_{}_{}_{}_{}",
            project_id.map(|id| id.to_string()).unwrap_or_else(|| "all".to_string()),
            limit.unwrap_or(25),
            offset.unwrap_or(0),
            include.as_ref().map(|i| i.join(",")).unwrap_or_else(|| "none".to_string()),
            easy_query_q.as_ref().unwrap_or(&"".to_string()),
            set_filter.unwrap_or(false),
            sort.as_ref().unwrap_or(&"".to_string()),
            assigned_to_id.unwrap_or(0),
            status_id.unwrap_or(0),
            tracker_id.unwrap_or(0),
            priority_id.unwrap_or(0)
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
            if let Some(query) = easy_query_q {
                query_params.push(("easy_query_q", query));
                // Pokud je easy_query_q zadáno, automaticky aktivujeme set_filter
                query_params.push(("set_filter", "1".to_string()));
            } else if let Some(true) = set_filter {
                query_params.push(("set_filter", "1".to_string()));
            }
            if let Some(sort) = sort {
                query_params.push(("sort", sort));
            }
            if let Some(assigned_to_id) = assigned_to_id {
                query_params.push(("assigned_to_id", assigned_to_id.to_string()));
            }
            if let Some(status_id) = status_id {
                query_params.push(("status_id", status_id.to_string()));
            }
            if let Some(tracker_id) = tracker_id {
                query_params.push(("tracker_id", tracker_id.to_string()));
            }
            if let Some(priority_id) = priority_id {
                query_params.push(("priority_id", priority_id.to_string()));
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
        
        // Pokud je odpověď prázdná, nejdříve získáme aktualizovaný úkol
        if response.as_object().map_or(false, |obj| obj.is_empty()) {
            debug!("Prázdná odpověď z update_issue, získávám aktualizovaný úkol");
            return self.get_issue(id, None).await;
        }
        
        self.parse_response(response)
    }

    // === USER API METHODS ===

    pub async fn list_users(&self, limit: Option<u32>, offset: Option<u32>, easy_query_q: Option<String>, set_filter: Option<bool>, sort: Option<String>, status: Option<String>) -> ApiResult<UsersResponse> {
        let cache_key = format!("users_{}_{}_{}_{}_{}",
            limit.unwrap_or(25),
            offset.unwrap_or(0),
            easy_query_q.as_ref().unwrap_or(&"".to_string()),
            set_filter.unwrap_or(false),
            sort.as_ref().unwrap_or(&"".to_string())
        );

        self.get_cached_or_fetch(&cache_key, "user", async {
            let url = format!("{}/users.json", self.base_url);
            let mut query_params = Vec::new();

            if let Some(limit) = limit {
                query_params.push(("limit", limit.to_string()));
            }
            if let Some(offset) = offset {
                query_params.push(("offset", offset.to_string()));
            }
            if let Some(query) = easy_query_q {
                query_params.push(("easy_query_q", query));
                // Pokud je easy_query_q zadáno, automaticky aktivujeme set_filter
                query_params.push(("set_filter", "1".to_string()));
            } else if let Some(true) = set_filter {
                query_params.push(("set_filter", "1".to_string()));
            }
            if let Some(sort) = sort {
                query_params.push(("sort", sort));
            }
            if let Some(status) = status {
                query_params.push(("status", status));
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

    pub async fn list_time_entries(&self, project_id: Option<i32>, user_id: Option<i32>, limit: Option<u32>, offset: Option<u32>, from_date: Option<String>, to_date: Option<String>) -> ApiResult<TimeEntriesResponse> {
        let cache_key = format!("time_entries_{}_{}_{}_{}_{}_{}",
            project_id.map(|id| id.to_string()).unwrap_or_else(|| "all".to_string()),
            user_id.map(|id| id.to_string()).unwrap_or_else(|| "all".to_string()),
            limit.unwrap_or(25), 
            offset.unwrap_or(0),
            from_date.as_ref().unwrap_or(&"none".to_string()),
            to_date.as_ref().unwrap_or(&"none".to_string())
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
            if let Some(from_date) = from_date {
                query_params.push(("from", from_date));
            }
            if let Some(to_date) = to_date {
                query_params.push(("to", to_date));
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

    // === MILESTONE (VERSION) API METHODS ===

    pub async fn list_milestones(&self, limit: Option<u32>, offset: Option<u32>, project_id: Option<i32>, status: Option<String>, easy_query_q: Option<String>) -> ApiResult<VersionsResponse> {
        let cache_key = format!("milestones_{}_{}_{}_{}_{}", 
            limit.unwrap_or(25),
            offset.unwrap_or(0),
            project_id.unwrap_or(0),
            status.as_ref().unwrap_or(&"all".to_string()),
            easy_query_q.as_ref().unwrap_or(&"".to_string())
        );

        self.get_cached_or_fetch(&cache_key, "milestone", async {
            let url = format!("{}/versions.json", self.base_url);
            let mut query_params = Vec::new();

            if let Some(limit) = limit {
                query_params.push(("limit", limit.to_string()));
            }
            if let Some(offset) = offset {
                query_params.push(("offset", offset.to_string()));
            }
            if let Some(status) = status {
                query_params.push(("status", status));
            }
            if let Some(query) = easy_query_q {
                query_params.push(("easy_query_q", query));
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

    pub async fn get_milestone(&self, id: i32) -> ApiResult<VersionResponse> {
        let cache_key = format!("milestone_{}", id);

        self.get_cached_or_fetch(&cache_key, "milestone", async {
            let url = format!("{}/versions/{}.json", self.base_url, id);
            let request = self.add_auth(self.http_client.get(&url));

            let response = self.execute_request(request).await?;
            self.parse_response(response)
        }).await
    }

    pub async fn create_milestone(
        &self,
        project_id: i32,
        name: String,
        description: Option<String>,
        effective_date: Option<String>,
        due_date: Option<String>,
        status: Option<String>,
        sharing: Option<String>,
        default_project_version: Option<bool>,
        easy_external_id: Option<String>,
    ) -> ApiResult<VersionResponse> {
        let url = format!("{}/projects/{}/versions.json", self.base_url, project_id);
        
        let create_version = CreateVersion {
            name,
            description,
            effective_date: effective_date.and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            due_date: due_date.and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            status,
            sharing,
            default_project_version,
            easy_external_id,
        };

        let request_body = CreateVersionRequest { version: create_version };
        let request = self.add_auth(self.http_client.post(&url))
            .json(&request_body);

        let response = self.execute_request(request).await?;
        
        // Invalidace cache
        self.invalidate_cache("milestone").await;
        
        self.parse_response(response)
    }

    pub async fn update_milestone(
        &self,
        id: i32,
        name: Option<String>,
        description: Option<String>,
        effective_date: Option<String>,
        due_date: Option<String>,
        status: Option<String>,
        sharing: Option<String>,
        default_project_version: Option<bool>,
        easy_external_id: Option<String>,
    ) -> ApiResult<VersionResponse> {
        let url = format!("{}/versions/{}.json", self.base_url, id);
        
        let update_version = UpdateVersion {
            name,
            description,
            effective_date: effective_date.and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            due_date: due_date.and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            status,
            sharing,
            default_project_version,
            easy_external_id,
        };

        let request_body = UpdateVersionRequest { version: update_version };
        let request = self.add_auth(self.http_client.put(&url))
            .json(&request_body);

        let response = self.execute_request(request).await?;
        
        // Invalidace cache
        self.invalidate_cache("milestone").await;
        
        self.parse_response(response)
    }

    pub async fn delete_milestone(&self, id: i32) -> ApiResult<()> {
        let url = format!("{}/versions/{}.json", self.base_url, id);
        let request = self.add_auth(self.http_client.delete(&url));

        let _response = self.execute_request(request).await?;
        
        // Invalidace cache
        self.invalidate_cache("milestone").await;
        
        Ok(())
    }

    // === ENUMERATION HELPER METHODS ===

    /// Interně získá číselníky pro issues pomocí paginace
    /// Skenuje issues a extrahuje všechny unikátní hodnoty pro status, priority, tracker
    pub async fn get_issue_enumerations(&self, project_id: Option<i32>) -> ApiResult<IssueEnumerationsResponse> {
        use std::collections::HashMap;

        debug!("Interně získávám číselníky pro issues, project_id: {:?}", project_id);

        let mut statuses: HashMap<i32, String> = HashMap::new();
        let mut priorities: HashMap<i32, String> = HashMap::new();
        let mut trackers: HashMap<i32, String> = HashMap::new();

        let mut offset = 0;
        let limit = 100;
        let max_iterations = 20; // Max 2000 issues pro skenování
        let mut iteration = 0;

        loop {
            iteration += 1;
            if iteration > max_iterations {
                debug!("Dosažen maximální počet iterací ({}) při skenování issues", max_iterations);
                break;
            }

            // Interně získáme stránku issues
            let response = self.list_issues(
                project_id,
                Some(limit),
                Some(offset),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None
            ).await?;

            if response.issues.is_empty() {
                debug!("Žádné další issues k zpracování");
                break;
            }

            // Extrahujeme číselníky z aktuální stránky
            for issue in &response.issues {
                statuses.insert(issue.status.id, issue.status.name.clone());
                priorities.insert(issue.priority.id, issue.priority.name.clone());
                trackers.insert(issue.tracker.id, issue.tracker.name.clone());
            }

            // Zkontrolujeme, jestli jsou další záznamy
            let total = response.total_count.unwrap_or(response.issues.len() as i32);
            offset += limit;

            if offset >= total as u32 {
                debug!("Zpracovány všechny issues ({})", total);
                break;
            }
        }

        // Převedeme HashMapy na seřazené Vec
        let mut status_list: Vec<_> = statuses.into_iter()
            .map(|(id, name)| EnumerationValue { id, name })
            .collect();
        status_list.sort_by_key(|v| v.id);

        let mut priority_list: Vec<_> = priorities.into_iter()
            .map(|(id, name)| EnumerationValue { id, name })
            .collect();
        priority_list.sort_by_key(|v| v.id);

        let mut tracker_list: Vec<_> = trackers.into_iter()
            .map(|(id, name)| EnumerationValue { id, name })
            .collect();
        tracker_list.sort_by_key(|v| v.id);

        info!("Získány číselníky: {} statusů, {} priorit, {} trackerů",
            status_list.len(), priority_list.len(), tracker_list.len());

        Ok(IssueEnumerationsResponse {
            statuses: status_list,
            priorities: priority_list,
            trackers: tracker_list,
        })
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