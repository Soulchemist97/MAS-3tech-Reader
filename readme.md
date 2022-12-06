
<p align="center"><img src="https://github.com/Soulchemist97/MAS3-Reader/blob/main/Logo/MAS_Reader_Logo.png?raw=true" height="350px"></p>



# **M**erkur **A**uslese **S**ystem (**MAS**-3 Tech)

Dateien aus dem MAS-3 Tech (.ACK) 
können mit diesem Tool ausgelesen, systematisch umbenannt, in PDFs umgewandelt und wichtige Daten in einer Excel-Tabelle ausgegeben werden.  

Beachte es gibt keinerlei direkte Verbindung zur Gauselmann Group und Ich übernehme keine Haftung für gelöschte Daten bei falscher Nutzung des Skripts.


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


| Aufstellort | Ausdruck_Nr | Zulassungsnummer | Geraetetyp | Anfangsdatum | Enddatum   | Ablaufdatum | Saldo1 | Saldo2 | Einsaetze | Gewinne |
|-------------|-------------|------------------|------------|--------------|------------|-------------|--------|--------|-----------|---------|
| Bar-Name    | A 008       | 123456789        | M. MULTI   | 01.01.2020   | 01.02.2020 | 2023/05     | 1215,4 | 1433,8 | 11059,3   | -9843,9 |
  

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

Liest Rechnungsdatei aus Pfad(../Dateiname.ACK) mit dem Angegebenen Aufstellort aus.


## Aufstellort
Klassenobjekt mit allen Rechnungsobjekten in Input/Aufstellort oder Alternative direkt über Pfad = Ordner

```
Ort1 = Aufstellort("Aufstellortname",Pfad=Ordnerpfad)
```
