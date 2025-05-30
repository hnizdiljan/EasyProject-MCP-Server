# Praktické příklady použití EasyProject MCP v Cursoru

## 🎯 Základní workflow

### 1. Analýza projektů

**Prompt:**
```
Zobraz mi přehled všech aktivních projektů včetně jejich stavu a termínů
```

**Co se stane:**
- MCP server zavolá `list_projects`
- Cursor zobrazí formátovaný seznam projektů
- Můžete pokračovat dalšími otázkami o konkrétních projektech

### 2. Správa úkolů

**Prompt:**
```
Vytvoř nový úkol v projektu "Website Redesign":
- Název: "Optimalizace obrázků pro web"
- Popis: "Zkomprimovat všechny obrázky pro rychlejší načítání"
- Přiřadit Janě Novákové
- Nastavit vysokou prioritu
- Odhadovaný čas: 4 hodiny
```

**Co se stane:**
1. Server najde projekt podle názvu
2. Najde uživatele podle jména
3. Vytvoří úkol s odpovídajícími parametry
4. Vrátí potvrzení s ID nového úkolu

### 3. Monitoring týmu

**Prompt:**
```
Zobraz mi pracovní vytížení všech vývojářů za poslední týden
```

**Co se stane:**
- Server získá seznam uživatelů
- Pro každého vývojáře spočítá pracovní vytížení
- Zobrazí přehled s odpracovanými hodinami a úkoly

## 📊 Reporting scénáře

### Projektové sestavy

**Prompt:**
```
Vygeneruj detailní sestavu projektu "Mobile App" za Q4 2023:
- Zahrň všechny úkoly a jejich stavy
- Zahrň časové záznamy
- Zahrň přehled týmu
```

**Výsledek:**
- Komprehensivní JSON sestava
- Grafy progress podle stavů úkolů
- Analýza času podle aktivit

### Dashboard data

**Prompt:**
```
Připrav dashboard data pro management:
- Pouze projekty s ID 1, 3, 5
- Za období od 1.11. do 30.11.2023
- Zahrň metriky dokončení a časových záznamů
```

## 🔄 Workflow automatizace

### Hromadné operace

**Prompt:**
```
Najdi všechny úkoly s prioritou "high" bez přiřazeného uživatele 
a přiřaď je vedoucímu týmu (ID: 5)
```

**Co se stane:**
1. Server získá všechny úkoly
2. Filtruje podle priority a nepřiřazené
3. Postupně přiřadí každý úkol vedoucímu
4. Vrátí souhrn změn

### Sledování deadline

**Prompt:**
```
Zobraz mi všechny úkoly, které mají termín dokončení do konce týdne 
a ještě nejsou dokončené
```

## 💡 Pokročilé použití

### Analýza produktivity

**Prompt:**
```
Pro projekt "Website Redesign" mi analyzuj:
1. Průměrný čas dokončení úkolu podle typu
2. Nejproduktivenější členy týmu
3. Úkoly s nejvyšším overheadem (rozdíl odhad vs. skutečnost)
```

### Plánování sprintů

**Prompt:**
```
Na základě historických dat a současného vytížení týmu 
navrhni optimální rozdělení úkolů pro příští sprint:
- Kapacita týmu: 80 hodin/týden
- Prioritní úkoly z backlogu
- Zohledni skillset jednotlivých členů
```

## 🎨 Integrace s kódem

### Vytváření úkolů z kódu

**Prompt (při práci s kódem):**
```
Na základě tohoto TODO komentáře v kódu vytvoř úkol v EasyProject:

// TODO: Refactorovat tuto funkci pro lepší performance
function processLargeDataset(data) {
    // neoptimální implementace
}

Projekt: "Backend Optimization"
Přiřadit: lead developer
```

### Code review workflow

**Prompt:**
```
Vytvoř úkol pro code review tohoto souboru:
- Název: "Code review: user-authentication.js"
- Popis: Zahrň aktuální změny a checklisty
- Přiřadit senior developerovi
- Odhadovaný čas: 1 hodina
```

## 📈 Monitorování metrik

### Týdenní reporty

**Prompt (každý pátek):**
```
Vygeneruj týdenní report pro celý tým:
- Dokončené úkoly za týden
- Celkové odpracované hodiny
- Top 3 nejproduktivnější členové
- Úkoly přesunuté na příští týden
```

### Burndown analýza

**Prompt:**
```
Pro projekt "Q1 Release" zobraz burndown data:
- Celkový počet úkolů vs. dokončené
- Trend dokončování za posledních 30 dní
- Predikce dokončení na základě současného tempa
```

## 🛠️ Troubleshooting s Cursor

### Debug problémů

**Prompt:**
```
Pomoz mi identifikovat, proč projekt "Mobile App" má zpoždění:
- Zobraz úkoly po termínu
- Najdi bottlenecky v týmu
- Identifikuj nejproblémovější oblasti
```

### Performance analýza

**Prompt:**
```
Analyzuj performance našeho projektového managementu:
- Průměrný čas od vytvoření po dokončení úkolu
- Projekty s nejvyšším počtem změn požadavků
- Nejčastější důvody prodlení
```

## 🔧 Kombinace s dalšími nástroji

### Git integrace

**Prompt:**
```
Na základě posledních commitů vytvoř time entry:
- Analyzuj git log za dnes
- Přiřaď čas k odpovídajícím úkolům v EasyProject
- Součet: 6 hodin práce
```

### Slack notifikace

**Prompt:**
```
Připrav Slack zprávu pro tým s dnešním progress:
- Dokončené úkoly
- Nové úkoly přiřazené
- Aktuální stav sprintu
```

---

**Tip:** Cursor si pamatuje kontext vaší konverzace, takže můžete navazovat dotazy typu:
- "Přidej k tomu poslednímu úkolu ještě tag 'urgent'"
- "Aktualizuj termín dokončení na příští týden"
- "Přiřaď podobný úkol i ostatním členům týmu" 