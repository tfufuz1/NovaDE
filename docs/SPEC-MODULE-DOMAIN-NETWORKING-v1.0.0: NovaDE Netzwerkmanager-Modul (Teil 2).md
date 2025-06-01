# SPEC-MODULE-DOMAIN-NETWORKING-v1.0.0: NovaDE Netzwerkmanager-Modul (Teil 2)

## 6. Datenmodell (Fortsetzung)

### 6.11 NetworkStatusType

```
ENTITÄT: NetworkStatusType
BESCHREIBUNG: Typ eines Netzwerkstatus
ATTRIBUTE:
  - NAME: status_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      Connection,
      Interface,
      Service,
      Protocol,
      Address,
      Route,
      Firewall,
      Proxy,
      Custom(String)
    }
    STANDARDWERT: Connection
INVARIANTEN:
  - Bei Custom darf die Zeichenkette nicht leer sein
```

### 6.12 NetworkEventType

```
ENTITÄT: NetworkEventType
BESCHREIBUNG: Typ eines Netzwerkereignisses
ATTRIBUTE:
  - NAME: event_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      Connection,
      Interface,
      Service,
      Protocol,
      Address,
      Route,
      Firewall,
      Proxy,
      Custom(String)
    }
    STANDARDWERT: Connection
INVARIANTEN:
  - Bei Custom darf die Zeichenkette nicht leer sein
```

### 6.13 NetworkConnectionState

```
ENTITÄT: NetworkConnectionState
BESCHREIBUNG: Zustand einer Netzwerkverbindung
ATTRIBUTE:
  - NAME: state
    TYP: Enum
    BESCHREIBUNG: Zustand
    WERTEBEREICH: {
      Unknown,
      Disconnected,
      Connecting,
      Connected,
      Disconnecting,
      Failed,
      Unavailable
    }
    STANDARDWERT: Unknown
INVARIANTEN:
  - Keine
```

### 6.14 NetworkInterfaceState

```
ENTITÄT: NetworkInterfaceState
BESCHREIBUNG: Zustand einer Netzwerkschnittstelle
ATTRIBUTE:
  - NAME: state
    TYP: Enum
    BESCHREIBUNG: Zustand
    WERTEBEREICH: {
      Unknown,
      Unmanaged,
      Unavailable,
      Disconnected,
      Preparing,
      ConfiguringHardware,
      NeedAuth,
      ConfiguringIp,
      CheckingIp,
      Secondaries,
      Activated,
      Deactivating,
      Failed
    }
    STANDARDWERT: Unknown
INVARIANTEN:
  - Keine
```

### 6.15 NetworkServiceState

```
ENTITÄT: NetworkServiceState
BESCHREIBUNG: Zustand eines Netzwerkdienstes
ATTRIBUTE:
  - NAME: state
    TYP: Enum
    BESCHREIBUNG: Zustand
    WERTEBEREICH: {
      Unknown,
      Stopped,
      Starting,
      Running,
      Stopping,
      Failed,
      Reloading,
      Inactive,
      Maintenance
    }
    STANDARDWERT: Unknown
INVARIANTEN:
  - Keine
```

### 6.16 NetworkConnectionInfo

```
ENTITÄT: NetworkConnectionInfo
BESCHREIBUNG: Informationen für eine Netzwerkverbindung
ATTRIBUTE:
  - NAME: id
    TYP: Option<String>
    BESCHREIBUNG: ID der Verbindung
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name der Verbindung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: type
    TYP: NetworkConnectionType
    BESCHREIBUNG: Typ der Verbindung
    WERTEBEREICH: Gültige NetworkConnectionType
    STANDARDWERT: NetworkConnectionType::Ethernet
  - NAME: interface_id
    TYP: Option<String>
    BESCHREIBUNG: ID der Schnittstelle
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
  - NAME: settings
    TYP: NetworkConnectionSettings
    BESCHREIBUNG: Einstellungen der Verbindung
    WERTEBEREICH: Gültige NetworkConnectionSettings
    STANDARDWERT: NetworkConnectionSettings::default()
  - NAME: autoconnect
    TYP: bool
    BESCHREIBUNG: Ob die Verbindung automatisch verbunden werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: is_default
    TYP: bool
    BESCHREIBUNG: Ob die Verbindung die Standardverbindung sein soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: timestamp
    TYP: Option<SystemTime>
    BESCHREIBUNG: Zeitpunkt der Erstellung
    WERTEBEREICH: Gültiger Zeitpunkt oder None
    STANDARDWERT: None
INVARIANTEN:
  - name darf nicht leer sein
```

### 6.17 NetworkConnectionSettings

```
ENTITÄT: NetworkConnectionSettings
BESCHREIBUNG: Einstellungen für eine Netzwerkverbindung
ATTRIBUTE:
  - NAME: ipv4
    TYP: Option<Ipv4Settings>
    BESCHREIBUNG: IPv4-Einstellungen
    WERTEBEREICH: Gültige Ipv4Settings oder None
    STANDARDWERT: None
  - NAME: ipv6
    TYP: Option<Ipv6Settings>
    BESCHREIBUNG: IPv6-Einstellungen
    WERTEBEREICH: Gültige Ipv6Settings oder None
    STANDARDWERT: None
  - NAME: wifi
    TYP: Option<WifiSettings>
    BESCHREIBUNG: WLAN-Einstellungen
    WERTEBEREICH: Gültige WifiSettings oder None
    STANDARDWERT: None
  - NAME: ethernet
    TYP: Option<EthernetSettings>
    BESCHREIBUNG: Ethernet-Einstellungen
    WERTEBEREICH: Gültige EthernetSettings oder None
    STANDARDWERT: None
  - NAME: mobile
    TYP: Option<MobileSettings>
    BESCHREIBUNG: Mobile-Einstellungen
    WERTEBEREICH: Gültige MobileSettings oder None
    STANDARDWERT: None
  - NAME: bluetooth
    TYP: Option<BluetoothSettings>
    BESCHREIBUNG: Bluetooth-Einstellungen
    WERTEBEREICH: Gültige BluetoothSettings oder None
    STANDARDWERT: None
  - NAME: vpn
    TYP: Option<VpnSettings>
    BESCHREIBUNG: VPN-Einstellungen
    WERTEBEREICH: Gültige VpnSettings oder None
    STANDARDWERT: None
  - NAME: dns
    TYP: Option<DnsSettings>
    BESCHREIBUNG: DNS-Einstellungen
    WERTEBEREICH: Gültige DnsSettings oder None
    STANDARDWERT: None
  - NAME: proxy
    TYP: Option<ProxySettings>
    BESCHREIBUNG: Proxy-Einstellungen
    WERTEBEREICH: Gültige ProxySettings oder None
    STANDARDWERT: None
  - NAME: security
    TYP: Option<SecuritySettings>
    BESCHREIBUNG: Sicherheitseinstellungen
    WERTEBEREICH: Gültige SecuritySettings oder None
    STANDARDWERT: None
  - NAME: custom
    TYP: HashMap<String, String>
    BESCHREIBUNG: Benutzerdefinierte Einstellungen
    WERTEBEREICH: Gültige Schlüssel-Wert-Paare
    STANDARDWERT: HashMap::new()
INVARIANTEN:
  - Mindestens eine der Einstellungen muss vorhanden sein
```

### 6.18 Ipv4Settings

```
ENTITÄT: Ipv4Settings
BESCHREIBUNG: IPv4-Einstellungen
ATTRIBUTE:
  - NAME: method
    TYP: Ipv4Method
    BESCHREIBUNG: Methode für die IPv4-Konfiguration
    WERTEBEREICH: Gültige Ipv4Method
    STANDARDWERT: Ipv4Method::Auto
  - NAME: addresses
    TYP: Vec<Ipv4Address>
    BESCHREIBUNG: IPv4-Adressen
    WERTEBEREICH: Gültige Ipv4Address-Werte
    STANDARDWERT: Leerer Vec
  - NAME: dns
    TYP: Vec<String>
    BESCHREIBUNG: DNS-Server
    WERTEBEREICH: Gültige IPv4-Adressen
    STANDARDWERT: Leerer Vec
  - NAME: dns_search
    TYP: Vec<String>
    BESCHREIBUNG: DNS-Suchdomänen
    WERTEBEREICH: Gültige Domänennamen
    STANDARDWERT: Leerer Vec
  - NAME: routes
    TYP: Vec<Ipv4Route>
    BESCHREIBUNG: IPv4-Routen
    WERTEBEREICH: Gültige Ipv4Route-Werte
    STANDARDWERT: Leerer Vec
  - NAME: may_fail
    TYP: bool
    BESCHREIBUNG: Ob die Verbindung fehlschlagen darf
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: never_default
    TYP: bool
    BESCHREIBUNG: Ob die Verbindung nie die Standardverbindung sein soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: dhcp_client_id
    TYP: Option<String>
    BESCHREIBUNG: DHCP-Client-ID
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
  - NAME: dhcp_timeout
    TYP: u32
    BESCHREIBUNG: DHCP-Timeout in Sekunden
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 45
  - NAME: dhcp_hostname
    TYP: Option<String>
    BESCHREIBUNG: DHCP-Hostname
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
  - NAME: dhcp_fqdn
    TYP: Option<String>
    BESCHREIBUNG: DHCP-FQDN
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
  - NAME: dhcp_send_hostname
    TYP: bool
    BESCHREIBUNG: Ob der Hostname an den DHCP-Server gesendet werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: ignore_auto_routes
    TYP: bool
    BESCHREIBUNG: Ob automatische Routen ignoriert werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: ignore_auto_dns
    TYP: bool
    BESCHREIBUNG: Ob automatische DNS-Server ignoriert werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
INVARIANTEN:
  - Wenn method == Ipv4Method::Manual, muss addresses nicht leer sein
  - dhcp_timeout muss größer als 0 sein
```

### 6.19 Ipv4Method

```
ENTITÄT: Ipv4Method
BESCHREIBUNG: Methode für die IPv4-Konfiguration
ATTRIBUTE:
  - NAME: method
    TYP: Enum
    BESCHREIBUNG: Methode
    WERTEBEREICH: {
      Auto,
      Manual,
      LinkLocal,
      Shared,
      Disabled
    }
    STANDARDWERT: Auto
INVARIANTEN:
  - Keine
```

### 6.20 Ipv4Address

```
ENTITÄT: Ipv4Address
BESCHREIBUNG: IPv4-Adresse
ATTRIBUTE:
  - NAME: address
    TYP: String
    BESCHREIBUNG: Adresse
    WERTEBEREICH: Gültige IPv4-Adresse
    STANDARDWERT: Keiner
  - NAME: prefix
    TYP: u8
    BESCHREIBUNG: Präfixlänge
    WERTEBEREICH: [0, 32]
    STANDARDWERT: 24
  - NAME: gateway
    TYP: Option<String>
    BESCHREIBUNG: Gateway
    WERTEBEREICH: Gültige IPv4-Adresse oder None
    STANDARDWERT: None
INVARIANTEN:
  - address muss eine gültige IPv4-Adresse sein
  - prefix muss im Bereich [0, 32] liegen
  - Wenn gateway vorhanden ist, muss es eine gültige IPv4-Adresse sein
```

## 7. Verhaltensmodell

### 7.1 Netzwerkverbindungsaktivierung

```
ZUSTANDSAUTOMAT: NetworkConnectionActivation
BESCHREIBUNG: Prozess der Aktivierung einer Netzwerkverbindung
ZUSTÄNDE:
  - NAME: Inactive
    BESCHREIBUNG: Verbindung ist inaktiv
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: Preparing
    BESCHREIBUNG: Verbindung wird vorbereitet
    EINTRITTSAKTIONEN: Verbindungsinformationen laden
    AUSTRITTSAKTIONEN: Keine
  - NAME: ConfiguringInterface
    BESCHREIBUNG: Schnittstelle wird konfiguriert
    EINTRITTSAKTIONEN: Schnittstellenkonfiguration starten
    AUSTRITTSAKTIONEN: Keine
  - NAME: NeedAuthentication
    BESCHREIBUNG: Authentifizierung wird benötigt
    EINTRITTSAKTIONEN: Authentifizierungsanfrage senden
    AUSTRITTSAKTIONEN: Keine
  - NAME: ConfiguringIp
    BESCHREIBUNG: IP wird konfiguriert
    EINTRITTSAKTIONEN: IP-Konfiguration starten
    AUSTRITTSAKTIONEN: Keine
  - NAME: CheckingIp
    BESCHREIBUNG: IP wird geprüft
    EINTRITTSAKTIONEN: IP-Prüfung starten
    AUSTRITTSAKTIONEN: Keine
  - NAME: ConfiguringDns
    BESCHREIBUNG: DNS wird konfiguriert
    EINTRITTSAKTIONEN: DNS-Konfiguration starten
    AUSTRITTSAKTIONEN: Keine
  - NAME: ConfiguringRoutes
    BESCHREIBUNG: Routen werden konfiguriert
    EINTRITTSAKTIONEN: Routenkonfiguration starten
    AUSTRITTSAKTIONEN: Keine
  - NAME: ConfiguringProxy
    BESCHREIBUNG: Proxy wird konfiguriert
    EINTRITTSAKTIONEN: Proxykonfiguration starten
    AUSTRITTSAKTIONEN: Keine
  - NAME: ConfiguringFirewall
    BESCHREIBUNG: Firewall wird konfiguriert
    EINTRITTSAKTIONEN: Firewallkonfiguration starten
    AUSTRITTSAKTIONEN: Keine
  - NAME: Active
    BESCHREIBUNG: Verbindung ist aktiv
    EINTRITTSAKTIONEN: Verbindungsstatus setzen
    AUSTRITTSAKTIONEN: Keine
  - NAME: Failed
    BESCHREIBUNG: Verbindung ist fehlgeschlagen
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Inactive
    NACH: Preparing
    EREIGNIS: activate_connection aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: Preparing
    NACH: ConfiguringInterface
    EREIGNIS: Vorbereitung erfolgreich
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: Preparing
    NACH: Failed
    EREIGNIS: Fehler bei der Vorbereitung
    BEDINGUNG: Keine
    AKTIONEN: NetworkError erstellen
  - VON: ConfiguringInterface
    NACH: NeedAuthentication
    EREIGNIS: Authentifizierung erforderlich
    BEDINGUNG: Verbindung erfordert Authentifizierung
    AKTIONEN: Keine
  - VON: ConfiguringInterface
    NACH: ConfiguringIp
    EREIGNIS: Schnittstellenkonfiguration erfolgreich
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ConfiguringInterface
    NACH: Failed
    EREIGNIS: Fehler bei der Schnittstellenkonfiguration
    BEDINGUNG: Keine
    AKTIONEN: NetworkError erstellen
  - VON: NeedAuthentication
    NACH: ConfiguringIp
    EREIGNIS: Authentifizierung erfolgreich
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: NeedAuthentication
    NACH: Failed
    EREIGNIS: Authentifizierung fehlgeschlagen
    BEDINGUNG: Keine
    AKTIONEN: NetworkError erstellen
  - VON: ConfiguringIp
    NACH: CheckingIp
    EREIGNIS: IP-Konfiguration erfolgreich
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ConfiguringIp
    NACH: Failed
    EREIGNIS: Fehler bei der IP-Konfiguration
    BEDINGUNG: !settings.ipv4.may_fail && !settings.ipv6.may_fail
    AKTIONEN: NetworkError erstellen
  - VON: CheckingIp
    NACH: ConfiguringDns
    EREIGNIS: IP-Prüfung erfolgreich
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: CheckingIp
    NACH: Failed
    EREIGNIS: Fehler bei der IP-Prüfung
    BEDINGUNG: !settings.ipv4.may_fail && !settings.ipv6.may_fail
    AKTIONEN: NetworkError erstellen
  - VON: ConfiguringDns
    NACH: ConfiguringRoutes
    EREIGNIS: DNS-Konfiguration erfolgreich
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ConfiguringDns
    NACH: Failed
    EREIGNIS: Fehler bei der DNS-Konfiguration
    BEDINGUNG: !settings.dns.may_fail
    AKTIONEN: NetworkError erstellen
  - VON: ConfiguringRoutes
    NACH: ConfiguringProxy
    EREIGNIS: Routenkonfiguration erfolgreich
    BEDINGUNG: settings.proxy.enabled
    AKTIONEN: Keine
  - VON: ConfiguringRoutes
    NACH: ConfiguringFirewall
    EREIGNIS: Routenkonfiguration erfolgreich
    BEDINGUNG: !settings.proxy.enabled && settings.firewall.enabled
    AKTIONEN: Keine
  - VON: ConfiguringRoutes
    NACH: Active
    EREIGNIS: Routenkonfiguration erfolgreich
    BEDINGUNG: !settings.proxy.enabled && !settings.firewall.enabled
    AKTIONEN: Keine
  - VON: ConfiguringRoutes
    NACH: Failed
    EREIGNIS: Fehler bei der Routenkonfiguration
    BEDINGUNG: Keine
    AKTIONEN: NetworkError erstellen
  - VON: ConfiguringProxy
    NACH: ConfiguringFirewall
    EREIGNIS: Proxykonfiguration erfolgreich
    BEDINGUNG: settings.firewall.enabled
    AKTIONEN: Keine
  - VON: ConfiguringProxy
    NACH: Active
    EREIGNIS: Proxykonfiguration erfolgreich
    BEDINGUNG: !settings.firewall.enabled
    AKTIONEN: Keine
  - VON: ConfiguringProxy
    NACH: Failed
    EREIGNIS: Fehler bei der Proxykonfiguration
    BEDINGUNG: !settings.proxy.may_fail
    AKTIONEN: NetworkError erstellen
  - VON: ConfiguringFirewall
    NACH: Active
    EREIGNIS: Firewallkonfiguration erfolgreich
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ConfiguringFirewall
    NACH: Failed
    EREIGNIS: Fehler bei der Firewallkonfiguration
    BEDINGUNG: !settings.firewall.may_fail
    AKTIONEN: NetworkError erstellen
INITIALZUSTAND: Inactive
ENDZUSTÄNDE: [Active, Failed]
```

### 7.2 Netzwerkverbindungsdeaktivierung

```
ZUSTANDSAUTOMAT: NetworkConnectionDeactivation
BESCHREIBUNG: Prozess der Deaktivierung einer Netzwerkverbindung
ZUSTÄNDE:
  - NAME: Active
    BESCHREIBUNG: Verbindung ist aktiv
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: Preparing
    BESCHREIBUNG: Deaktivierung wird vorbereitet
    EINTRITTSAKTIONEN: Verbindungsinformationen laden
    AUSTRITTSAKTIONEN: Keine
  - NAME: RemovingFirewall
    BESCHREIBUNG: Firewallregeln werden entfernt
    EINTRITTSAKTIONEN: Firewallregeln entfernen
    AUSTRITTSAKTIONEN: Keine
  - NAME: RemovingProxy
    BESCHREIBUNG: Proxykonfiguration wird entfernt
    EINTRITTSAKTIONEN: Proxykonfiguration entfernen
    AUSTRITTSAKTIONEN: Keine
  - NAME: RemovingRoutes
    BESCHREIBUNG: Routen werden entfernt
    EINTRITTSAKTIONEN: Routen entfernen
    AUSTRITTSAKTIONEN: Keine
  - NAME: RemovingDns
    BESCHREIBUNG: DNS-Konfiguration wird entfernt
    EINTRITTSAKTIONEN: DNS-Konfiguration entfernen
    AUSTRITTSAKTIONEN: Keine
  - NAME: RemovingIp
    BESCHREIBUNG: IP-Konfiguration wird entfernt
    EINTRITTSAKTIONEN: IP-Konfiguration entfernen
    AUSTRITTSAKTIONEN: Keine
  - NAME: DeconfiguringInterface
    BESCHREIBUNG: Schnittstelle wird dekonfiguriert
    EINTRITTSAKTIONEN: Schnittstellenkonfiguration entfernen
    AUSTRITTSAKTIONEN: Keine
  - NAME: Inactive
    BESCHREIBUNG: Verbindung ist inaktiv
    EINTRITTSAKTIONEN: Verbindungsstatus setzen
    AUSTRITTSAKTIONEN: Keine
  - NAME: Failed
    BESCHREIBUNG: Deaktivierung ist fehlgeschlagen
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Active
    NACH: Preparing
    EREIGNIS: deactivate_connection aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: Preparing
    NACH: RemovingFirewall
    EREIGNIS: Vorbereitung erfolgreich
    BEDINGUNG: settings.firewall.enabled
    AKTIONEN: Keine
  - VON: Preparing
    NACH: RemovingProxy
    EREIGNIS: Vorbereitung erfolgreich
    BEDINGUNG: !settings.firewall.enabled && settings.proxy.enabled
    AKTIONEN: Keine
  - VON: Preparing
    NACH: RemovingRoutes
    EREIGNIS: Vorbereitung erfolgreich
    BEDINGUNG: !settings.firewall.enabled && !settings.proxy.enabled
    AKTIONEN: Keine
  - VON: Preparing
    NACH: Failed
    EREIGNIS: Fehler bei der Vorbereitung
    BEDINGUNG: Keine
    AKTIONEN: NetworkError erstellen
  - VON: RemovingFirewall
    NACH: RemovingProxy
    EREIGNIS: Firewallregeln erfolgreich entfernt
    BEDINGUNG: settings.proxy.enabled
    AKTIONEN: Keine
  - VON: RemovingFirewall
    NACH: RemovingRoutes
    EREIGNIS: Firewallregeln erfolgreich entfernt
    BEDINGUNG: !settings.proxy.enabled
    AKTIONEN: Keine
  - VON: RemovingFirewall
    NACH: Failed
    EREIGNIS: Fehler beim Entfernen der Firewallregeln
    BEDINGUNG: Keine
    AKTIONEN: NetworkError erstellen
  - VON: RemovingProxy
    NACH: RemovingRoutes
    EREIGNIS: Proxykonfiguration erfolgreich entfernt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: RemovingProxy
    NACH: Failed
    EREIGNIS: Fehler beim Entfernen der Proxykonfiguration
    BEDINGUNG: Keine
    AKTIONEN: NetworkError erstellen
  - VON: RemovingRoutes
    NACH: RemovingDns
    EREIGNIS: Routen erfolgreich entfernt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: RemovingRoutes
    NACH: Failed
    EREIGNIS: Fehler beim Entfernen der Routen
    BEDINGUNG: Keine
    AKTIONEN: NetworkError erstellen
  - VON: RemovingDns
    NACH: RemovingIp
    EREIGNIS: DNS-Konfiguration erfolgreich entfernt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: RemovingDns
    NACH: Failed
    EREIGNIS: Fehler beim Entfernen der DNS-Konfiguration
    BEDINGUNG: Keine
    AKTIONEN: NetworkError erstellen
  - VON: RemovingIp
    NACH: DeconfiguringInterface
    EREIGNIS: IP-Konfiguration erfolgreich entfernt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: RemovingIp
    NACH: Failed
    EREIGNIS: Fehler beim Entfernen der IP-Konfiguration
    BEDINGUNG: Keine
    AKTIONEN: NetworkError erstellen
  - VON: DeconfiguringInterface
    NACH: Inactive
    EREIGNIS: Schnittstelle erfolgreich dekonfiguriert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: DeconfiguringInterface
    NACH: Failed
    EREIGNIS: Fehler bei der Dekonfiguration der Schnittstelle
    BEDINGUNG: Keine
    AKTIONEN: NetworkError erstellen
INITIALZUSTAND: Active
ENDZUSTÄNDE: [Inactive, Failed]
```

## 8. Fehlerbehandlung

### 8.1 Fehlerbehandlungsstrategie

1. Alle Fehler MÜSSEN über spezifische Fehlertypen zurückgegeben werden.
2. Fehlertypen MÜSSEN mit `thiserror` definiert werden.
3. Fehler MÜSSEN kontextuelle Informationen enthalten.
4. Fehlerketten MÜSSEN bei der Weitergabe oder beim Wrappen von Fehlern erhalten bleiben.
5. Panics sind VERBOTEN, außer in Fällen, die explizit dokumentiert sind.

### 8.2 Modulspezifische Fehlertypen

```
ENTITÄT: NetworkError
BESCHREIBUNG: Fehler im Netzwerkmanager-Modul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      ConnectionNotFound { id: String },
      InterfaceNotFound { id: String },
      ServiceNotFound { id: String },
      InvalidConnection { id: String, message: String },
      InvalidInterface { id: String, message: String },
      InvalidService { id: String, message: String },
      ConnectionActivationError { id: String, message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      ConnectionDeactivationError { id: String, message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      InterfaceEnableError { id: String, message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      InterfaceDisableError { id: String, message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      ServiceStartError { id: String, message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      ServiceStopError { id: String, message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      FirewallError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      ProxyError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      DnsError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      RouteError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      AddressError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      ScanError { interface_id: String, message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      ConfigError { message: String },
      IoError { message: String, source: std::io::Error },
      DBusError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

## 9. Leistungsanforderungen

### 9.1 Allgemeine Leistungsanforderungen

1. Das Netzwerkmanager-Modul MUSS effizient mit Ressourcen umgehen.
2. Das Netzwerkmanager-Modul MUSS eine geringe Latenz haben.
3. Das Netzwerkmanager-Modul MUSS skalierbar sein.

### 9.2 Spezifische Leistungsanforderungen

1. Das Aktivieren einer Netzwerkverbindung MUSS in unter 5 Sekunden abgeschlossen sein (ohne Berücksichtigung der Verbindungszeit).
2. Das Deaktivieren einer Netzwerkverbindung MUSS in unter 3 Sekunden abgeschlossen sein.
3. Das Abrufen von Netzwerkinformationen MUSS in unter 100ms abgeschlossen sein.
4. Das Suchen nach verfügbaren Netzwerken MUSS in unter 5 Sekunden abgeschlossen sein.
5. Das Netzwerkmanager-Modul MUSS mit mindestens 100 Netzwerkverbindungen umgehen können.
6. Das Netzwerkmanager-Modul MUSS mit mindestens 20 Netzwerkschnittstellen umgehen können.
7. Das Netzwerkmanager-Modul DARF nicht mehr als 1% CPU-Auslastung im Leerlauf verursachen.
8. Das Netzwerkmanager-Modul DARF nicht mehr als 50MB Speicher verbrauchen.

## 10. Sicherheitsanforderungen

### 10.1 Allgemeine Sicherheitsanforderungen

1. Das Netzwerkmanager-Modul MUSS memory-safe sein.
2. Das Netzwerkmanager-Modul MUSS thread-safe sein.
3. Das Netzwerkmanager-Modul MUSS robust gegen Fehleingaben sein.

### 10.2 Spezifische Sicherheitsanforderungen

1. Das Netzwerkmanager-Modul MUSS Eingaben validieren, um Command Injection-Angriffe zu verhindern.
2. Das Netzwerkmanager-Modul MUSS Zugriffskontrollen für Netzwerkoperationen implementieren.
3. Das Netzwerkmanager-Modul MUSS sichere Standardwerte verwenden.
4. Das Netzwerkmanager-Modul MUSS Ressourcenlimits implementieren, um Denial-of-Service-Angriffe zu verhindern.
5. Das Netzwerkmanager-Modul MUSS verhindern, dass nicht autorisierte Anwendungen auf geschützte Netzwerkressourcen zugreifen.
6. Das Netzwerkmanager-Modul MUSS Netzwerkänderungen protokollieren.
7. Das Netzwerkmanager-Modul MUSS Netzwerkberechtigungen sicher verwalten.
8. Das Netzwerkmanager-Modul MUSS Netzwerkressourcen überwachen und begrenzen.

## 11. Testkriterien

### 11.1 Allgemeine Testkriterien

1. Jede Komponente MUSS Einheitstests haben.
2. Jede öffentliche Funktion MUSS getestet sein.
3. Jeder Fehlerfall MUSS getestet sein.

### 11.2 Spezifische Testkriterien

1. Das Netzwerkmanager-Modul MUSS mit verschiedenen Netzwerkverbindungstypen getestet sein.
2. Das Netzwerkmanager-Modul MUSS mit verschiedenen Netzwerkschnittstellentypen getestet sein.
3. Das Netzwerkmanager-Modul MUSS mit verschiedenen Netzwerkdiensttypen getestet sein.
4. Das Netzwerkmanager-Modul MUSS mit verschiedenen Netzwerkprotokolltypen getestet sein.
5. Das Netzwerkmanager-Modul MUSS mit verschiedenen Netzwerkadresstypen getestet sein.
6. Das Netzwerkmanager-Modul MUSS mit verschiedenen Netzwerkroutentypen getestet sein.
7. Das Netzwerkmanager-Modul MUSS mit verschiedenen Netzwerkfirewalltypen getestet sein.
8. Das Netzwerkmanager-Modul MUSS mit verschiedenen Netzwerkproxytypen getestet sein.
9. Das Netzwerkmanager-Modul MUSS mit verschiedenen Netzwerkmonitortypen getestet sein.
10. Das Netzwerkmanager-Modul MUSS mit verschiedenen Netzwerkstatustypen getestet sein.
11. Das Netzwerkmanager-Modul MUSS mit verschiedenen Netzwerkereignistypen getestet sein.
12. Das Netzwerkmanager-Modul MUSS mit verschiedenen Fehlerszenarien getestet sein.
13. Das Netzwerkmanager-Modul MUSS mit verschiedenen Benutzerinteraktionen getestet sein.
14. Das Netzwerkmanager-Modul MUSS mit vielen gleichzeitigen Netzwerkverbindungen getestet sein.

## 12. Anhänge

### 12.1 Referenzierte Dokumente

1. SPEC-ROOT-v1.0.0: NovaDE Spezifikationswurzel
2. SPEC-LAYER-CORE-v1.0.0: Spezifikation der Kernschicht
3. SPEC-LAYER-DOMAIN-v1.0.0: Spezifikation der Domänenschicht
4. SPEC-MODULE-SYSTEM-NOTIFICATION-v1.0.0: Spezifikation des Benachrichtigungsmanager-Moduls
5. SPEC-MODULE-DOMAIN-SETTINGS-v1.0.0: Spezifikation des Einstellungsmanager-Moduls
6. SPEC-MODULE-SYSTEM-SECURITY-v1.0.0: Spezifikation des Sicherheitsmanager-Moduls
7. SPEC-MODULE-SYSTEM-POWER-v1.0.0: Spezifikation des Energieverwaltungsmanager-Moduls
8. SPEC-MODULE-SYSTEM-USER-v1.0.0: Spezifikation des Benutzerverwaltungsmanager-Moduls
9. SPEC-MODULE-SYSTEM-FILESYSTEM-v1.0.0: Spezifikation des Dateisystemmanager-Moduls

### 12.2 Externe Abhängigkeiten

1. `libnm`: Für die Netzwerkverwaltung
2. `dbus`: Für die D-Bus-Integration
3. `serde`: Für die Serialisierung und Deserialisierung
4. `json`: Für die JSON-Verarbeitung
5. `async-std`: Für asynchrone Operationen
6. `futures`: Für Future-basierte Programmierung
7. `tokio`: Für asynchrone I/O-Operationen
