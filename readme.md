# **M**erkur **A**uslese **S**ystem (**MAS**-3 Tech)

Dateien aus dem MAS-3 Tech (.ACK) 
können mit diesem Tool ausgelesen, systematisch umbenannt und in PDF umgewandelt werden und wichtige Daten in einer Excel-Tabelle ausgegeben.  

## Features: 
- Automatische Bennennung der Rechnungen in: 
  
```
Aufstellort [Anfangsdatum-Auslesedatum](A #Ausdrucknr (#Zulassungsnr).ACK
```

### Nutzung des Skripts:
1. (.ACK)-Dateien im Ordner: Input/Aufstellort ablegen
2. Skript ausführen und Input Fragen ausfüllen:
   1. Dateiinfos im Terminal ausgegeben.
   2. Verschieben, kopieren oder Nichts:
   - Verschieben: Verschiebt Dateien umbenannt in den Ordner Output und überschreibt doppelte Rechnungen
   - Kopieren: Kopiert die Dateien in umbenannter Form in Output, aber gibt Fehler bei doppelten Quittungen
   - Nichts: Indiziert Dateien für Excel Ausgabe ohne Verschiebung oder Umbennenung 
   3.  Save as Excel-File: Speichern aller Infos in Excel-Tabellen im Ordner Zusammenfassungen/Aufstellort.xlsx
   4.  Save Database: Speichert alle Aufstellortobjekte als Pickle Datei ab. (Experimentell bisher) 


## Rechnung

Klassenobjekt welches folgende Eigenschaften beinhaltet:

- Anfangs- und End-Datum
- Ausdruck Nr.
- Zulassungsnummer
- Saldo 1 & 2
- Einsaetze
- Gewinne
- Gerätetyp


Codebeispiel:
```
Quittung = Rechnung("Aufstellortname","Pfad")
```

Liest Rechnunsdatei aus Aufstellort/Dateiname.ACK aus.


## Aufstellort
Klassenobjekt mit allen Rechnungsobjekten in Input/Aufstellort

```
Ort1 = Aufstellort("Aufstellortname")
```
