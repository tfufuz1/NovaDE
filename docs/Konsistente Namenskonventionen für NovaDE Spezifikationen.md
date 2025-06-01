# Konsistente Namenskonventionen für NovaDE Spezifikationen

## 1. Dokumentidentifikation

### 1.1 Spezifikationsdokumente
- **Format**: `SPEC-{TYP}-{NAME}-v{X.Y.Z}`
- **Beispiele**:
  - `SPEC-LAYER-CORE-v1.0.0`: Spezifikation der Kernschicht
  - `SPEC-MODULE-THEMING-v2.1.3`: Spezifikation des Theming-Moduls
  - `SPEC-INTERFACE-WORKSPACE_MANAGER-v1.5.0`: Spezifikation der Workspace-Manager-Schnittstelle

### 1.2 Anforderungsdokumente
- **Format**: `REQ-{BEREICH}-{ID}`
- **Beispiele**:
  - `REQ-CORE-001`: Anforderung an die Kernschicht
  - `REQ-UI-042`: Anforderung an die Benutzeroberfläche

### 1.3 Architekturentscheidungen
- **Format**: `ADR-{JAHR}{MONAT}-{ID}`
- **Beispiele**:
  - `ADR-202505-001`: Architekturentscheidung vom Mai 2025

## 2. Modulare Namensräume

### 2.1 Schichtennamensräume
- **Kernschicht**: `core`
- **Domänenschicht**: `domain`
- **Systemschicht**: `system`
- **UI-Schicht**: `ui`

### 2.2 Modulnamensräume
- **Format**: `{SCHICHT}::{MODUL}`
- **Beispiele**:
  - `core::types`: Typenmodul in der Kernschicht
  - `domain::theming`: Theming-Modul in der Domänenschicht
  - `system::compositor`: Compositor-Modul in der Systemschicht
  - `ui::shell`: Shell-Modul in der UI-Schicht

### 2.3 Komponentennamensräume
- **Format**: `{SCHICHT}::{MODUL}::{KOMPONENTE}`
- **Beispiele**:
  - `core::types::geometry`: Geometrie-Komponente im Typenmodul
  - `domain::theming::engine`: Engine-Komponente im Theming-Modul

## 3. Entitätsbezeichner

### 3.1 Schnittstellen (Traits)
- **Format**: PascalCase mit beschreibendem Suffix
- **Beispiele**:
  - `ThemingEngine`: Schnittstelle für die Theming-Engine
  - `WorkspaceManagerService`: Schnittstelle für den Workspace-Manager
  - `ConfigProvider`: Schnittstelle für Konfigurationsanbieter

### 3.2 Implementierungen
- **Format**: PascalCase mit `Default`- oder spezifischem Präfix
- **Beispiele**:
  - `DefaultThemingEngine`: Standardimplementierung der Theming-Engine
  - `FileSystemConfigProvider`: Dateisystembasierte Implementierung des ConfigProvider

### 3.3 Datenstrukturen
- **Format**: PascalCase für Strukturen und Enums
- **Beispiele**:
  - `WorkspaceId`: Identifikator für Workspaces
  - `NotificationUrgency`: Enum für Benachrichtigungsdringlichkeit
  - `ThemeDefinition`: Struktur für Themendefinitionen

### 3.4 Ereignisse
- **Format**: PascalCase mit `Event`-Suffix
- **Beispiele**:
  - `ThemeChangedEvent`: Ereignis bei Themenänderung
  - `WorkspaceCreatedEvent`: Ereignis bei Workspace-Erstellung
  - `SettingChangedEvent`: Ereignis bei Einstellungsänderung

### 3.5 Fehlertypen
- **Format**: PascalCase mit `Error`-Suffix
- **Beispiele**:
  - `CoreError`: Basisfehlertypus der Kernschicht
  - `ThemingError`: Fehler im Theming-Modul
  - `ConfigError`: Fehler in der Konfigurationsverwaltung

## 4. Versionierungskonventionen

### 4.1 Semantische Versionierung
- **Hauptversion (X)**: Inkompatible API-Änderungen
- **Nebenversion (Y)**: Abwärtskompatible Funktionserweiterungen
- **Patch (Z)**: Abwärtskompatible Fehlerbehebungen

### 4.2 Versionskennzeichnung in Spezifikationen
- **Format**: `v{X.Y.Z}`
- **Beispiele**:
  - `v1.0.0`: Initiale Version
  - `v1.2.3`: Version mit Fehlerbehebungen nach Funktionserweiterungen

### 4.3 Versionsverlauf
- **Format**: Tabellarische Darstellung mit Datum, Version, Autor und Änderungsbeschreibung
- **Beispiel**:
  ```
  | Datum      | Version | Autor | Änderungen                                |
  |------------|---------|-------|-------------------------------------------|
  | 2025-05-31 | v1.0.0  | LWJ   | Initiale Version                          |
  | 2025-06-15 | v1.1.0  | LWJ   | Erweiterung um Farbunterstützung          |
  | 2025-06-20 | v1.1.1  | LWJ   | Fehlerbehebung in Farbkonvertierung       |
  ```

## 5. Terminologische Konsistenz

### 5.1 Glossar
- Zentrales Glossar für alle Projekte
- Eindeutige Definition aller Fachbegriffe
- Konsistente Verwendung in allen Dokumenten

### 5.2 Abkürzungen
- Vollständige Erklärung bei erster Verwendung
- Konsistente Verwendung in allen Dokumenten

### 5.3 Modalverben
- **MUSS**: Zwingende Anforderung
- **SOLLTE**: Empfohlene Anforderung
- **KANN**: Optionale Anforderung

## 6. Dateibenennung

### 6.1 Spezifikationsdateien
- **Format**: `{typ}_{name}_v{x}_{y}_{z}.md`
- **Beispiele**:
  - `layer_core_v1_0_0.md`: Spezifikation der Kernschicht
  - `module_theming_v2_1_3.md`: Spezifikation des Theming-Moduls

### 6.2 Unterstützende Dokumente
- **Format**: `{typ}_{name}.{erweiterung}`
- **Beispiele**:
  - `glossary_novade.md`: Glossar für NovaDE
  - `dependency_matrix_v1_0.md`: Abhängigkeitsmatrix Version 1.0

## 7. Anwendung der Konventionen

### 7.1 Neue Dokumente
- Strikte Anwendung aller Konventionen
- Validierung vor Freigabe

### 7.2 Bestehende Dokumente
- Schrittweise Migration zu neuen Konventionen
- Klare Kennzeichnung des Migrationsstatus

### 7.3 Automatisierte Prüfung
- Linting-Tools für Namenskonventionen
- Validierungsskripte für Dokumentstruktur
- Konsistenzprüfungen für Querverweise
