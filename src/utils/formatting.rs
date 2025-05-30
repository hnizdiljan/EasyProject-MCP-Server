use chrono::{DateTime, Utc, NaiveDate};
use crate::api::models::{Project, Issue, User, TimeEntry, ProjectStatus};

/// Formátuje projekt pro lidsky čitelný výstup
pub fn format_project(project: &Project) -> String {
    let status = match project.status {
        ProjectStatus::Active => "Aktivní",
        ProjectStatus::Closed => "Uzavřený", 
        ProjectStatus::Archived => "Archivovaný",
        ProjectStatus::Planned => "Plánovaný",
        ProjectStatus::Deleted => "Smazaný",
        ProjectStatus::Unknown(status_id) => &format!("Neznámý ({})", status_id),
    };
    
    let mut result = format!(
        "Projekt #{}: {}\n  Status: {}\n",
        project.id,
        project.name,
        status
    );
    
    if let Some(ref description) = project.description {
        result.push_str(&format!("  Popis: {}\n", description));
    }
    
    if let Some(ref identifier) = project.identifier {
        result.push_str(&format!("  Identifikátor: {}\n", identifier));
    }
    
    if let Some(ref homepage) = project.homepage {
        result.push_str(&format!("  Domovská stránka: {}\n", homepage));
    }
    
    if let Some(ref parent) = project.parent {
        result.push_str(&format!("  Nadřazený projekt: {} (ID: {})\n", parent.name, parent.id));
    }
    
    if let Some(ref created_on) = project.created_on {
        result.push_str(&format!("  Vytvořeno: {}\n", format_datetime(created_on)));
    }
    
    result
}

/// Formátuje úkol pro lidsky čitelný výstup
pub fn format_issue(issue: &Issue) -> String {
    let mut result = format!(
        "Úkol #{}: {}\n  Projekt: {}\n  Tracker: {}\n  Status: {}\n  Priorita: {}\n",
        issue.id,
        issue.subject,
        issue.project.name,
        issue.tracker.name,
        issue.status.name,
        issue.priority.name
    );
    
    if let Some(ref description) = issue.description {
        let truncated = if description.len() > 200 {
            format!("{}...", &description[..200])
        } else {
            description.clone()
        };
        result.push_str(&format!("  Popis: {}\n", truncated));
    }
    
    if let Some(ref author) = issue.author {
        result.push_str(&format!("  Autor: {}\n", author.name));
    }
    
    if let Some(ref assigned_to) = issue.assigned_to {
        result.push_str(&format!("  Přiřazeno: {}\n", assigned_to.name));
    }
    
    if let Some(estimated_hours) = issue.estimated_hours {
        result.push_str(&format!("  Odhadované hodiny: {}\n", estimated_hours));
    }
    
    if let Some(spent_hours) = issue.spent_hours {
        result.push_str(&format!("  Strávené hodiny: {}\n", spent_hours));
    }
    
    if let Some(done_ratio) = issue.done_ratio {
        result.push_str(&format!("  Dokončeno: {}%\n", done_ratio));
    }
    
    if let Some(ref start_date) = issue.start_date {
        result.push_str(&format!("  Datum zahájení: {}\n", format_date(start_date)));
    }
    
    if let Some(ref due_date) = issue.due_date {
        result.push_str(&format!("  Termín dokončení: {}\n", format_date(due_date)));
    }
    
    if let Some(ref created_on) = issue.created_on {
        result.push_str(&format!("  Vytvořeno: {}\n", format_datetime(created_on)));
    }
    
    result
}

/// Formátuje uživatele pro lidsky čitelný výstup
pub fn format_user(user: &User) -> String {
    let status = match user.status {
        Some(1) => "Aktivní",
        Some(2) => "Registrovaný",
        Some(3) => "Zablokovaný",
        _ => "Neznámý",
    };
    
    let firstname = user.firstname.as_deref().unwrap_or("N/A");
    let lastname = user.lastname.as_deref().unwrap_or("N/A");
    
    let mut result = format!(
        "Uživatel #{}: {} {}\n  Status: {}\n",
        user.id,
        firstname,
        lastname,
        status
    );
    
    if let Some(ref login) = user.login {
        result.push_str(&format!("  Přihlašovací jméno: {}\n", login));
    }
    
    if let Some(ref mail) = user.mail {
        result.push_str(&format!("  Email: {}\n", mail));
    }
    
    if let Some(admin) = user.admin {
        if admin {
            result.push_str("  Role: Administrátor\n");
        }
    }
    
    if let Some(ref created_on) = user.created_on {
        result.push_str(&format!("  Vytvořeno: {}\n", format_datetime(created_on)));
    }
    
    if let Some(ref last_login_on) = user.last_login_on {
        result.push_str(&format!("  Poslední přihlášení: {}\n", format_datetime(last_login_on)));
    }
    
    result
}

/// Formátuje časový záznam pro lidsky čitelný výstup
pub fn format_time_entry(time_entry: &TimeEntry) -> String {
    let mut result = format!(
        "Časový záznam #{}: {} hodin\n  Projekt: {}\n  Aktivita: {}\n  Datum: {}\n  Uživatel: {}\n",
        time_entry.id,
        time_entry.hours,
        time_entry.project.name,
        time_entry.activity.name,
        format_date(&time_entry.spent_on),
        time_entry.user.name
    );
    
    if let Some(ref issue) = time_entry.issue {
        result.push_str(&format!("  Úkol: #{}\n", issue.id));
    }
    
    if let Some(ref comments) = time_entry.comments {
        result.push_str(&format!("  Komentář: {}\n", comments));
    }
    
    if let Some(ref created_on) = time_entry.created_on {
        result.push_str(&format!("  Vytvořeno: {}\n", format_datetime(created_on)));
    }
    
    result
}

/// Formátuje DateTime pro výstup
pub fn format_datetime(datetime: &DateTime<Utc>) -> String {
    datetime.format("%d.%m.%Y %H:%M:%S UTC").to_string()
}

/// Formátuje NaiveDate pro výstup
pub fn format_date(date: &NaiveDate) -> String {
    date.format("%d.%m.%Y").to_string()
}

/// Formátuje seznam projektů pro přehled
pub fn format_project_list(projects: &[Project]) -> String {
    if projects.is_empty() {
        return "Žádné projekty nebyly nalezeny.".to_string();
    }
    
    let mut result = format!("Nalezeno {} projektů:\n\n", projects.len());
    
    for project in projects {
        let status = match project.status {
            ProjectStatus::Active => "Aktivní",
            ProjectStatus::Closed => "Uzavřený",
            ProjectStatus::Archived => "Archivovaný",
            ProjectStatus::Planned => "Plánovaný",
            ProjectStatus::Deleted => "Smazaný",
            ProjectStatus::Unknown(status_id) => &format!("Neznámý ({})", status_id),
        };
        
        result.push_str(&format!(
            "• #{}: {} ({})\n",
            project.id,
            project.name,
            status
        ));
        
        if let Some(ref description) = project.description {
            let truncated = if description.len() > 100 {
                format!("{}...", &description[..100])
            } else {
                description.clone()
            };
            result.push_str(&format!("  {}\n", truncated));
        }
        
        result.push('\n');
    }
    
    result
}

/// Formátuje seznam úkolů pro přehled
pub fn format_issue_list(issues: &[Issue]) -> String {
    if issues.is_empty() {
        return "Žádné úkoly nebyly nalezeny.".to_string();
    }
    
    let mut result = format!("Nalezeno {} úkolů:\n\n", issues.len());
    
    for issue in issues {
        result.push_str(&format!(
            "• #{}: {} [{}]\n",
            issue.id,
            issue.subject,
            issue.status.name
        ));
        
        result.push_str(&format!(
            "  Projekt: {} | Priorita: {}\n",
            issue.project.name,
            issue.priority.name
        ));
        
        if let Some(ref assigned_to) = issue.assigned_to {
            result.push_str(&format!("  Přiřazeno: {}\n", assigned_to.name));
        }
        
        if let Some(done_ratio) = issue.done_ratio {
            result.push_str(&format!("  Dokončeno: {}%\n", done_ratio));
        }
        
        result.push('\n');
    }
    
    result
}

/// Formátuje seznam uživatelů pro přehled
pub fn format_user_list(users: &[User]) -> String {
    if users.is_empty() {
        return "Žádní uživatelé nebyli nalezeni.".to_string();
    }
    
    let mut result = format!("Nalezeno {} uživatelů:\n\n", users.len());
    
    for user in users {
        let status = match user.status {
            Some(1) => "Aktivní",
            Some(2) => "Registrovaný", 
            Some(3) => "Zablokovaný",
            _ => "Neznámý",
        };
        
        let firstname = user.firstname.as_deref().unwrap_or("N/A");
        let lastname = user.lastname.as_deref().unwrap_or("N/A");
        
        result.push_str(&format!(
            "• #{}: {} {} ({})\n",
            user.id,
            firstname,
            lastname,
            status
        ));
        
        if let Some(ref mail) = user.mail {
            result.push_str(&format!("  Email: {}\n", mail));
        }
        
        if let Some(admin) = user.admin {
            if admin {
                result.push_str("  Role: Administrátor\n");
            }
        }
        
        result.push('\n');
    }
    
    result
}

/// Formátuje seznam časových záznamů pro přehled
pub fn format_time_entry_list(time_entries: &[TimeEntry]) -> String {
    if time_entries.is_empty() {
        return "Žádné časové záznamy nebyly nalezeny.".to_string();
    }
    
    let mut result = format!("Nalezeno {} časových záznamů:\n\n", time_entries.len());
    let total_hours: f64 = time_entries.iter().map(|te| te.hours).sum();
    
    for time_entry in time_entries {
        result.push_str(&format!(
            "• #{}: {} hodin - {} ({})\n",
            time_entry.id,
            time_entry.hours,
            time_entry.project.name,
            format_date(&time_entry.spent_on)
        ));
        
        result.push_str(&format!(
            "  Aktivita: {} | Uživatel: {}\n",
            time_entry.activity.name,
            time_entry.user.name
        ));
        
        if let Some(ref issue) = time_entry.issue {
            result.push_str(&format!("  Úkol: #{}\n", issue.id));
        }
        
        if let Some(ref comments) = time_entry.comments {
            let truncated = if comments.len() > 80 {
                format!("{}...", &comments[..80])
            } else {
                comments.clone()
            };
            result.push_str(&format!("  Komentář: {}\n", truncated));
        }
        
        result.push('\n');
    }
    
    result.push_str(&format!("Celkem hodin: {}\n", total_hours));
    
    result
}

/// Formátuje chybovou zprávu
pub fn format_error(error: &str) -> String {
    format!("❌ Chyba: {}", error)
}

/// Formátuje úspěšnou zprávu
pub fn format_success(message: &str) -> String {
    format!("✅ {}", message)
}

/// Formátuje informační zprávu
pub fn format_info(message: &str) -> String {
    format!("ℹ️ {}", message)
}

/// Formátuje varovnou zprávu
pub fn format_warning(message: &str) -> String {
    format!("⚠️ {}", message)
} 